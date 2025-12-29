use crate::backend::{
    providers::hf::{API_URL, HF_URL},
    settings::get_settings,
};
use axum::extract::Path;
use axum::response::IntoResponse;
use axum_streams::StreamBodyAs;
use futures::{Stream, StreamExt};
use ochat_types::providers::hf::{HFModelDetails, HFPullModelResponse, HFPullModelStreamResult};
use reqwest::header::RANGE;
use std::path::PathBuf;
use std::{collections::HashMap, time::Duration};
use tokio::fs;
use tokio::io::AsyncWriteExt;

const EXTRA_FILES: [&str; 1] = ["tokenizer.json"];
const MAX_RETRIES: u32 = 5;

async fn run_pull_stream(
    user: String,
    model: String,
    name: String,
) -> impl Stream<Item = HFPullModelStreamResult> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap_or_default();

    let details_res = client
        .get(format!(
            "{}/models/{}/{}",
            API_URL,
            user.trim(),
            model.trim()
        ))
        .send()
        .await;

    if details_res.is_err() {
        let _ = tx.send(HFPullModelStreamResult::Err(
            "Failed to fetch model details".to_string(),
        ));
        let _ = tx.send(HFPullModelStreamResult::Finished);
        return Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx));
    }

    let details: HFModelDetails = match details_res.unwrap().json().await {
        Ok(d) => d,
        Err(_) => {
            let _ = tx.send(HFPullModelStreamResult::Err(
                "Failed to parse model details".to_string(),
            ));
            let _ = tx.send(HFPullModelStreamResult::Finished);
            return Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx));
        }
    };

    let model_path = get_models_dir(
        format!("{}/{}", user, model),
        if let Some(tag) = details.pipeline_tag {
            if tag.contains("speech") || tag.contains("recognition") {
                "speech"
            } else {
                "text"
            }
        } else {
            "text"
        }
        .to_string(),
    )
    .await;

    let bin_path = model_path.join(name.trim());
    let model_ref = if let Some(x) = details.base_model.clone() {
        x
    } else {
        format!("{}/{}", user.trim(), model.trim())
    };

    let base_url = format!("https://huggingface.co/{}/resolve/main/", model_ref.trim());

    let mut files: Vec<(String, PathBuf)> = Vec::new();

    files.push((
        format!(
            "{}/{}/{}/resolve/main/{}?download=true",
            HF_URL,
            user.trim(),
            model.trim(),
            name.trim()
        ),
        bin_path.clone(),
    ));

    for required in EXTRA_FILES {
        files.push((
            format!("{}{}", base_url, required),
            model_path.join(required),
        ));
    }

    let file_count = files.len();
    let (p_tx, mut p_rx) = tokio::sync::mpsc::unbounded_channel::<(usize, Option<u64>, u64)>();
    for (i, (url, path)) in files.into_iter().enumerate() {
        let p_tx = p_tx.clone();
        let path = path.clone();

        tokio::spawn(async move {
            let temp_path = path.with_extension("tmp");
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent).await;
            }

            let mut attempt = 0;

            loop {
                attempt += 1;

                let mut start_offset = 0u64;
                if temp_path.exists() {
                    if let Ok(meta) = fs::metadata(&temp_path).await {
                        start_offset = meta.len();
                    }
                }

                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(60))
                    .build()
                    .unwrap_or_default();

                let mut req_builder = client.get(&url);
                if start_offset > 0 {
                    req_builder = req_builder.header(RANGE, format!("bytes={}-", start_offset));
                }

                let resp = match req_builder.send().await {
                    Ok(r) => r,
                    Err(_) => {
                        if attempt >= MAX_RETRIES {
                            let _ = p_tx.send((i, None, 0));
                            return;
                        }
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                };

                let status = resp.status();
                if !status.is_success() {
                    if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
                        let _ = fs::remove_file(&temp_path).await;
                        continue;
                    }

                    if status.is_client_error() {
                        if temp_path.exists() {
                            let _ = fs::remove_file(&temp_path).await;
                        }
                        let _ = p_tx.send((i, Some(1), 1));
                        return;
                    }

                    if attempt >= MAX_RETRIES {
                        let _ = p_tx.send((i, None, 0));
                        return;
                    }
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }

                let mut out_file = match fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(true)
                    .open(&temp_path)
                    .await
                {
                    Ok(f) => f,
                    Err(_) => {
                        let _ = p_tx.send((i, None, 0));
                        return;
                    }
                };

                let content_len = resp.content_length().unwrap_or(0);
                let total_size = if start_offset > 0 {
                    Some(start_offset + content_len)
                } else {
                    resp.content_length()
                };

                let mut stream = resp.bytes_stream();
                let mut current_pos = start_offset;
                let mut download_failed = false;

                while let Some(chunk_res) = stream.next().await {
                    match chunk_res {
                        Ok(chunk) => {
                            if out_file.write_all(&chunk).await.is_err() {
                                download_failed = true;
                                break;
                            }
                            current_pos += chunk.len() as u64;
                            let _ = p_tx.send((i, total_size, current_pos));
                        }
                        Err(_) => {
                            download_failed = true;
                            break;
                        }
                    }
                }

                let _ = out_file.flush().await;

                if !download_failed {
                    if fs::rename(&temp_path, &path).await.is_ok() {
                        let final_total = total_size.or(Some(current_pos));
                        let _ = p_tx.send((i, final_total, current_pos));
                        return;
                    }
                }

                if attempt >= MAX_RETRIES {
                    let _ = p_tx.send((i, None, 0));
                    return;
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });
    }

    tokio::spawn(async move {
        let mut totals: HashMap<usize, Option<u64>> = HashMap::new();
        let mut completeds: HashMap<usize, u64> = HashMap::new();

        while let Some((idx, total_opt, completed)) = p_rx.recv().await {
            totals.insert(idx, total_opt);
            completeds.insert(idx, completed);

            let mut sum_percent = 0f64;
            for i in 0..file_count {
                let comp = *completeds.get(&i).unwrap_or(&0u64);
                let tot_opt = *totals.get(&i).unwrap_or(&None);

                let p = if let Some(tot) = tot_opt {
                    if tot == 0 {
                        1.0
                    } else {
                        comp as f64 / tot as f64
                    }
                } else {
                    0f64
                };
                sum_percent += p;
            }

            let avg = sum_percent / (file_count as f64);
            let completed_val = (avg * 100.0).clamp(0.0, 100.0) as u64;

            let _ = tx.send(HFPullModelStreamResult::Pulling(HFPullModelResponse {
                total: Some(100),
                completed: Some(completed_val),
            }));
        }

        let _ = tx.send(HFPullModelStreamResult::Finished);
    });

    Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx))
}

#[axum::debug_handler]
pub async fn run(Path((user, id, name)): Path<(String, String, String)>) -> impl IntoResponse {
    StreamBodyAs::json_nl(run_pull_stream(user, id, name).await)
}

pub async fn get_models_dir(model: String, sub_type: String) -> PathBuf {
    let dir = get_settings().await.unwrap().models_path.clone();
    let (user, model) = model.rsplit_once("/").unwrap();

    if !fs::try_exists(&dir).await.unwrap_or(true) {
        let _ = fs::create_dir(&dir).await.unwrap();
    }

    let mut model_path = dir.join(sub_type.trim());

    if !fs::try_exists(&model_path).await.unwrap_or(true) {
        let _ = fs::create_dir(&model_path).await.unwrap();
    }

    model_path = model_path.join(user.trim());

    if !fs::try_exists(&model_path).await.unwrap_or(true) {
        let _ = fs::create_dir(&model_path).await.unwrap();
    }

    model_path = model_path.join(model.trim());

    if !fs::try_exists(&model_path).await.unwrap_or(true) {
        let _ = fs::create_dir(&model_path).await.unwrap();
    }

    model_path
}
