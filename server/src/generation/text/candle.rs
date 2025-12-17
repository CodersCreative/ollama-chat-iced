use axum::Json;
use futures::Stream;
use ochat_types::generation::text::{ChatQueryData, ChatResponse, ChatStreamResult};
use tokio_stream::StreamExt;

use crate::errors::ServerError;

pub async fn run(data: ChatQueryData) -> Result<Json<ChatResponse>, ServerError> {
    todo!()
}

pub async fn stream(data: ChatQueryData) -> impl Stream<Item = ChatStreamResult> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    return Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx));
}
