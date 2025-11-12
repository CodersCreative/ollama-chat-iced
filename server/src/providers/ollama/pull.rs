use crate::providers::Provider;
use crate::{CONN, providers::PROVIDER_TABLE};
use axum::extract::Path;
use axum::response::IntoResponse;
use axum_streams::StreamBodyAs;
use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_stream::StreamExt;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PullModelStreamResult {
    Err(String),
    Pulling(PullModelResponse),
    Finished,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PullModelResponse {
    pub status: String,
    pub digest: Option<String>,
    pub total: Option<u64>,
    pub completed: Option<u64>,
}

async fn run_pull_stream(
    provider: String,
    model: String,
) -> impl Stream<Item = PullModelStreamResult> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    let mut response = if let Some(provider) = CONN
        .select::<Option<Provider>>((PROVIDER_TABLE, &*provider))
        .await
        .unwrap()
    {
        let client = reqwest::Client::new();
        let url = format!(
            "{}/api/pull",
            provider.url.trim_end_matches('/').trim_end_matches("/v1")
        );
        let builder = client.post(url);
        builder
            .json(&json!({
                "name" : model,
                "insecure" : true,
                "stream" : true
            }))
            .send()
            .await
            .unwrap()
            .bytes_stream()
    } else {
        panic!()
    };

    tokio::spawn(async move {
        while let Some(response) = response.next().await {
            match response {
                Ok(response) => {
                    let _ = match serde_json::from_slice::<PullModelResponse>(&response) {
                        Ok(x) => tx.send(PullModelStreamResult::Pulling(x)),
                        Err(e) => tx.send(PullModelStreamResult::Err(e.to_string())),
                    };
                }
                Err(e) => {
                    let _ = tx.send(PullModelStreamResult::Err(e.to_string()));
                }
            }
        }

        let _ = tx.send(PullModelStreamResult::Finished);
    });

    return Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx));
}

#[axum::debug_handler]
pub async fn run(Path((id, model)): Path<(String, String)>) -> impl IntoResponse {
    StreamBodyAs::json_nl(run_pull_stream(id, model).await)
}
