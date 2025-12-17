use crate::providers::hf::{API_URL, HF_URL};
use crate::settings::get_settings;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum_streams::StreamBodyAs;
use futures::{Stream, StreamExt};
use ochat_types::providers::hf::{HFModelDetails, HFPullModelResponse, HFPullModelStreamResult};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

const EXTRA_FILES: [&str; 1] = ["tokenizer.json"];

async fn run_pull_stream(
    dir: PathBuf,
    user: String,
    model: String,
    name: String,
) -> impl Stream<Item = HFPullModelStreamResult> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    if !fs::try_exists(&dir).await.unwrap_or(true) {
        let _ = fs::create_dir(&dir).await.unwrap();
    }

    let mut model_path = dir.join(user.trim());

    if !fs::try_exists(&model_path).await.unwrap_or(true) {
        let _ = fs::create_dir(&model_path).await.unwrap();
    }

    model_path = model_path.join(model.trim());

    if !fs::try_exists(&model_path).await.unwrap_or(true) {
        let _ = fs::create_dir(&model_path).await.unwrap();
    }

    let bin_path = model_path.join(name.trim());
    let details_res = reqwest::get(format!(
        "{}/models/{}/{}",
        API_URL,
        user.trim(),
        model.trim()
    ))
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
        let temp = path.clone().with_extension("tmp");
        tokio::spawn(async move {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent).await;
            }

            let mut out = match fs::File::create(&temp).await {
                Ok(f) => f,
                Err(_) => {
                    let _ = p_tx.send((i, None, 0));
                    return;
                }
            };

            let resp = match reqwest::get(&url).await {
                Ok(r) => r,
                Err(_) => {
                    let _ = p_tx.send((i, None, 0));
                    return;
                }
            };

            let total = resp.content_length();
            let mut completed: u64 = 0;

            let mut stream = resp.bytes_stream();

            while let Some(chunk_res) = stream.next().await {
                match chunk_res {
                    Ok(chunk) => {
                        if let Err(_) = out.write_all(&chunk).await {
                            let _ = p_tx.send((i, total, completed));
                            return;
                        }
                        completed += chunk.len() as u64;
                        let _ = p_tx.send((i, total, completed));
                    }
                    Err(_) => {
                        let _ = p_tx.send((i, total, completed));
                        return;
                    }
                }
            }

            let _ = out.flush().await;
            let _ = fs::rename(&temp, &path).await;
            let _ = p_tx.send((i, total, completed));
        });
    }

    tokio::spawn(async move {
        let mut totals: HashMap<usize, Option<u64>> = HashMap::new();
        let mut completeds: HashMap<usize, u64> = HashMap::new();
        let mut finished_flags: HashMap<usize, bool> = HashMap::new();

        while let Some((idx, total_opt, completed)) = p_rx.recv().await {
            totals.insert(idx, total_opt);
            completeds.insert(idx, completed);

            let mut sum_percent = 0f64;
            for i in 0..file_count {
                if let Some(tot_opt) = totals.get(&i) {
                    if let Some(tot) = tot_opt {
                        let comp = *completeds.get(&i).unwrap_or(&0u64);
                        let p = if *tot == 0 {
                            0f64
                        } else {
                            comp as f64 / *tot as f64
                        };
                        sum_percent += p;
                    } else {
                        sum_percent += 0f64;
                    }
                } else {
                    sum_percent += 0f64;
                }
            }

            let avg = sum_percent / (file_count as f64);
            let completed_val = (avg * 100.0).min(100.0).max(0.0) as u64;

            let _ = tx.send(HFPullModelStreamResult::Pulling(HFPullModelResponse {
                total: Some(100),
                completed: Some(completed_val),
            }));

            if let Some(tot_opt) = totals.get(&idx) {
                if let Some(tot) = tot_opt {
                    if let Some(c) = completeds.get(&idx) {
                        if *c >= *tot {
                            finished_flags.insert(idx, true);
                        }
                    }
                }
            }

            if finished_flags.len() >= file_count {
                let _ = tx.send(HFPullModelStreamResult::Finished);
                break;
            }
        }
    });

    return Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx));
}

#[axum::debug_handler]
pub async fn run(Path((user, id, name)): Path<(String, String, String)>) -> impl IntoResponse {
    let dir = get_settings().await.unwrap().models_path.clone();
    StreamBodyAs::json_nl(run_pull_stream(dir, user, id, name).await)
}
