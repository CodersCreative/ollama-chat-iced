use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Database Error : {0}")]
    Surreal(#[from] surrealdb::Error),
    #[error("Message Error : {0}")]
    RigMessageError(#[from] rig::completion::MessageError),
    #[error("Completion Error : {0}")]
    RigCompletionError(#[from] rig::completion::CompletionError),
    #[error("Prompt Error : {0}")]
    RigPromptError(#[from] rig::completion::PromptError),
    #[error("Embed Error : {0}")]
    RigEmbedError(#[from] rig::embeddings::EmbedError),
    #[error("Embedding Error : {0}")]
    RigEmbeddingError(#[from] rig::embeddings::EmbeddingError),
    #[error("Reqwest Error : {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("IO Error : {0}")]
    IO(#[from] std::io::Error),
    #[error("Error : {0}")]
    Unknown(String),
}

impl From<Box<dyn std::error::Error>> for ServerError {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Self::Unknown(value.to_string())
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self.to_string())).into_response()
    }
}
