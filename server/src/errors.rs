use async_openai::error::OpenAIError;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Database Error : {0}")]
    Surreal(surrealdb::Error),
    #[error("Generation Error : {0}")]
    OpenAI(OpenAIError),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self.to_string())).into_response()
    }
}

impl From<OpenAIError> for ServerError {
    fn from(error: OpenAIError) -> Self {
        eprintln!("{error}");
        Self::OpenAI(error)
    }
}
impl From<surrealdb::Error> for ServerError {
    fn from(error: surrealdb::Error) -> Self {
        eprintln!("{error}");
        Self::Surreal(error)
    }
}
