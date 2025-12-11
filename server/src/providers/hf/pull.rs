use crate::providers::hf::HF_URL;
use crate::settings::get_settings;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum_streams::StreamBodyAs;
use futures::Stream;
use ochat_types::providers::hf::{HFPullModelResponse, HFPullModelStreamResult};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::{AsyncWriteExt, BufWriter};

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

    let mut model_path = dir.join(&user);

    if !fs::try_exists(&model_path).await.unwrap_or(true) {
        let _ = fs::create_dir(&model_path).await.unwrap();
        model_path = model_path.join(&model);
    }

    if !fs::try_exists(&model_path).await.unwrap_or(true) {
        let _ = fs::create_dir(&model_path).await.unwrap();
        model_path = model_path.join(&name);
    }

    let temp_path = model_path.clone().with_extension("tmp");
    let mut file = BufWriter::new(fs::File::create(&temp_path).await.unwrap());

    let mut download = reqwest::get(format!(
        "{}/{}/{}/resolve/main/{}?download=true",
        HF_URL, user, model, name
    ))
    .await
    .unwrap();
    let total = download.content_length();
    let mut completed = 0;

    tokio::spawn(async move {
        while let Some(response) = download.chunk().await.unwrap() {
            completed += response.len() as u64;

            tx.send(HFPullModelStreamResult::Pulling(HFPullModelResponse {
                total: total,
                completed: Some(completed),
            }))
            .unwrap();

            let _ = file.write_all(&response).await.unwrap();
        }

        let _ = file.flush().await.unwrap();
        let _ = fs::rename(temp_path, model_path).await;
        let _ = tx.send(HFPullModelStreamResult::Finished);
    });

    return Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx));
}

#[axum::debug_handler]
pub async fn run(Path((user, id, name)): Path<(String, String, String)>) -> impl IntoResponse {
    let dir = get_settings().await.unwrap().models_path.clone();
    StreamBodyAs::json_nl(run_pull_stream(dir, user, id, name).await)
}
