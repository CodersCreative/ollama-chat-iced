use crate::backend::{
    providers::hf::{
        API_URL, HF_URL,
        conversion::{ModelFormat, convert_model},
        hf_auth_headers, hf_has_auth, model_is_gated,
    },
    settings::get_settings,
};
use axum::extract::Path;
use axum::response::IntoResponse;
use axum_streams::StreamBodyAs;
use futures::{FutureExt, Stream, StreamExt, future::join_all};
use ochat_types::providers::hf::{HFModelDetails, HFPullModelResponse, HFPullModelStreamResult};
use reqwest::header::RANGE;
use serde::Deserialize;
use std::{collections::HashMap, thread, time::Duration};
use std::{path::PathBuf, time::Instant};
use tokio::io::AsyncWriteExt;
use tokio::{
    fs::{self},
    time::timeout,
};

const EXTRA_FILES: [&str; 9] = [
    "tokenizer.json",
    "tokenizer.model",
    "tokenizer_config.json",
    "config.json",
    "preprocessor_config.json",
    "generation_config.json",
    "processor_config.json",
    "config_sentence_transformers.json",
    "chat_template.jinja",
];

pub const EXTRA_EXTS: [&str; 6] = ["json", "jinja", "tmp", "sec", "new", "ter"];
const MAX_RETRIES: u32 = 128;
const RETRY_RESET_TIME: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Deserialize)]
struct Entry {
    path: String,
}

pub async fn run_pull_stream(
    user: String,
    model: String,
    name: String,
    output_format: ModelFormat,
    sub_type: &str,
) -> impl Stream<Item = HFPullModelStreamResult> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let client = reqwest::Client::builder()
        .default_headers(hf_auth_headers().await)
        .timeout(REQUEST_TIMEOUT)
        .build()
        .unwrap_or_default();

    let model_id = format!("{}/{}", user.trim(), model.trim());

    let details_res = client
        .get(format!("{}/models/{}", API_URL, model_id))
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

    let entries_url = format!("{}/models/{}/tree/main", API_URL, model_id);
    let mut download_queue: Vec<(String, PathBuf)> = Vec::new();

    let model_path = get_models_dir(name.clone(), model_id.clone(), sub_type.to_string()).await;

    if let Ok(resp) = client.get(&entries_url).send().await {
        if let Ok(entries) = resp.json::<Vec<Entry>>().await {
            let is_safetensor = name.ends_with(".safetensors") && name.contains("-of-");

            if is_safetensor {
                let base_name = name.split('-').next().unwrap_or(&name);
                for entry in entries {
                    if entry.path.starts_with(base_name) && entry.path.ends_with(".safetensors") {
                        download_queue.push((
                            format!("{}/{}/resolve/main/{}", HF_URL, model_id, entry.path),
                            model_path.join(&entry.path),
                        ));
                    }
                }
            } else {
                download_queue.push((
                    format!("{}/{}/resolve/main/{}", HF_URL, model_id, name.trim()),
                    model_path.join(&name),
                ));
            }
        }
    }

    let mut base_model = details.get_base_model();

    base_model = if let Some(base_model) = base_model {
        if model_is_gated(&base_model, &client).await.unwrap_or(true) && !hf_has_auth().await {
            None
        } else {
            Some(base_model)
        }
    } else {
        None
    };

    let secondary_model = {
        let id = model_id.rsplit_once("-");
        if let Some(id) = id {
            if model_is_gated(id.0, &client).await.unwrap_or(true) && !hf_has_auth().await {
                None
            } else {
                Some(id.0.to_string())
            }
        } else {
            None
        }
    };

    for extra in EXTRA_FILES {
        if let Some(sec) = &secondary_model {
            download_queue.push((
                format!("{}/{}/resolve/main/{}", HF_URL, sec, extra),
                model_path.join(format!("{}.ter", extra)),
            ));
        }

        download_queue.push((
            format!("{}/{}/resolve/main/{}", HF_URL, model_id, extra),
            model_path.join(format!("{}.sec", extra)),
        ));

        if let Some(base_model) = &base_model {
            download_queue.push((
                format!("{}/{}/resolve/main/{}", HF_URL, base_model, extra),
                model_path.join(extra),
            ));
        }
    }

    let file_count = download_queue.len();
    let (p_tx, mut p_rx) = tokio::sync::mpsc::unbounded_channel::<(usize, Option<u64>, u64)>();

    let mut handles = Vec::new();
    for (i, (url, path)) in download_queue.into_iter().enumerate() {
        let p_tx = p_tx.clone();
        let client_clone = client.clone();
        handles.push(tokio::spawn(download_file(
            client_clone,
            url,
            path,
            i,
            p_tx,
        )));
    }

    tokio::spawn(async move {
        let mut totals: HashMap<usize, Option<u64>> = HashMap::new();
        let mut completeds: HashMap<usize, u64> = HashMap::new();
        let start_time = Instant::now();

        let mut all_downloads_fut = join_all(handles).fuse();
        let mut download_error = None;

        loop {
            tokio::select! {
                Some((idx, total_opt, completed)) = p_rx.recv() => {
                    totals.insert(idx, total_opt);
                    completeds.insert(idx, completed);

                    let mut sum_percent = 0.0;
                    let mut total_bytes = 0;
                    for i in 0..file_count {
                        let comp = *completeds.get(&i).unwrap_or(&0);
                        total_bytes += comp;
                        let tot = totals.get(&i).and_then(|t| *t).unwrap_or(comp.max(1));
                        sum_percent += comp as f64 / tot as f64;
                    }

                    let avg_progress = (sum_percent / file_count as f64) * 100.0;
                    let speed = total_bytes as f64 / start_time.elapsed().as_secs_f64().max(0.1);

                    let _ = tx.send(HFPullModelStreamResult::Pulling(HFPullModelResponse {
                        total: Some(100),
                        completed: Some(avg_progress as u64),
                        speed: Some(speed),
                    }));
                }
                results = &mut all_downloads_fut => {
                    for res in results {
                        match res {
                            Ok(Err(e)) => download_error = Some(e),
                            Err(join_err) => download_error = Some(format!("Task panic: {}", join_err)),
                            _ => {}
                        }
                    }
                    break;
                }
            }
        }

        if let Some(err_msg) = download_error {
            let _ = tx.send(HFPullModelStreamResult::Err(err_msg));
            return;
        }

        let mut updated = Vec::new();

        for path in model_path.read_dir().unwrap() {
            let path = path.unwrap().path();
            let file_size = fs::metadata(path.clone()).await.unwrap().len();

            if file_size <= 200 {
                let _ = fs::remove_file(&path).await;
                continue;
            }

            let desired = path
                .to_str()
                .unwrap()
                .rsplit_once(".")
                .unwrap()
                .0
                .to_string();

            if path.extension().unwrap().to_str().unwrap() == "sec" {
                if file_size > 1000 {
                    let _ = fs::rename(&path, &desired).await;
                    updated.push(desired);
                } else {
                    let _ = fs::remove_file(&path).await;
                }
            } else if path.extension().unwrap().to_str().unwrap() == "ter" {
                if file_size > 1000 && !updated.contains(&desired) {
                    let _ = fs::rename(&path, desired).await;
                } else {
                    let _ = fs::remove_file(&path).await;
                }
            }
        }

        match convert_model(&model_path, output_format).await {
            Ok(_) => {
                let _ = tx.send(HFPullModelStreamResult::Finished);
            }
            Err(e) => {
                let _ = tx.send(HFPullModelStreamResult::Err(e));
            }
        }
    });

    Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx))
}

async fn download_file(
    client: reqwest::Client,
    url: String,
    path: PathBuf,
    index: usize,
    p_tx: tokio::sync::mpsc::UnboundedSender<(usize, Option<u64>, u64)>,
) -> Result<(), String> {
    let temp_path = path.with_extension("tmp");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| e.to_string())?;
    }

    let mut attempts = 0;
    let mut last = Instant::now();
    while attempts < MAX_RETRIES {
        match attempt_download_stream(&client, &url, &temp_path, index, &p_tx).await {
            Ok(_) => {
                fs::rename(&temp_path, &path)
                    .await
                    .map_err(|e| e.to_string())?;
                return Ok(());
            }
            Err(e) => {
                if last.elapsed() >= RETRY_RESET_TIME {
                    attempts = 1;
                } else {
                    attempts += 1;
                }

                last = Instant::now();
                if attempts >= MAX_RETRIES {
                    return Err(format!(
                        "Download failed after {} attempts: {}",
                        MAX_RETRIES, e
                    ));
                }
                let backoff = Duration::from_secs(attempts as u64);
                thread::sleep(backoff);
            }
        }
    }
    Err("Unknown error during download".into())
}

async fn attempt_download_stream(
    client: &reqwest::Client,
    url: &str,
    temp_path: &PathBuf,
    index: usize,
    p_tx: &tokio::sync::mpsc::UnboundedSender<(usize, Option<u64>, u64)>,
) -> Result<(), String> {
    let start_offset = if temp_path.exists() {
        fs::metadata(temp_path).await.map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    let resp = timeout(
        REQUEST_TIMEOUT,
        client
            .get(url)
            .header(RANGE, format!("bytes={}-", start_offset))
            .send(),
    )
    .await
    .map_err(|_| "Connection timed out".to_string())?
    .map_err(|e| e.to_string())?;

    let total_size = if start_offset > 0 {
        resp.content_length().map(|l| l + start_offset)
    } else {
        resp.content_length()
    };

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .write(true)
        .open(temp_path)
        .await
        .map_err(|e| e.to_string())?;

    let mut stream = resp.bytes_stream();
    let mut current_pos = start_offset;

    while let Some(item) = timeout(REQUEST_TIMEOUT, stream.next())
        .await
        .map_err(|_| "Stream stalled")?
    {
        let chunk = item.map_err(|e| e.to_string())?;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        current_pos += chunk.len() as u64;
        let _ = p_tx.send((index, total_size, current_pos));
    }

    file.flush().await.map_err(|e| e.to_string())?;

    if let Some(expected) = total_size {
        if current_pos != expected {
            return Err(format!(
                "Size verification failed: expected {}, got {}",
                expected, current_pos
            ));
        }
    }

    Ok(())
}

#[axum::debug_handler]
pub async fn run_tts(Path((user, id, name)): Path<(String, String, String)>) -> impl IntoResponse {
    StreamBodyAs::json_nl(run_pull_stream(user, id, name, ModelFormat::SafeTensors, "tts").await)
}

#[axum::debug_handler]
pub async fn run_text(Path((user, id, name)): Path<(String, String, String)>) -> impl IntoResponse {
    StreamBodyAs::json_nl(run_pull_stream(user, id, name, ModelFormat::SafeTensors, "text").await)
}

#[axum::debug_handler]
pub async fn run_stt(Path((user, id, name)): Path<(String, String, String)>) -> impl IntoResponse {
    StreamBodyAs::json_nl(run_pull_stream(user, id, name, ModelFormat::GGML, "stt").await)
}

pub async fn get_models_dir(name: String, model: String, sub_type: String) -> PathBuf {
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

    let name = if let Some(x) = name.rsplit_once(".") {
        x.0.to_string()
    } else {
        name
    };

    model_path = model_path.join(name.trim());

    if !fs::try_exists(&model_path).await.unwrap_or(true) {
        let _ = fs::create_dir(&model_path).await.unwrap();
    }

    model_path
}
