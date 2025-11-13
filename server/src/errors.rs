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
    #[error("Generation Error : {0}")]
    OpenAI(#[from] async_openai::error::OpenAIError),
    #[error("Reqwest Error : {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Error : {0}")]
    Unknown(String),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self.to_string())).into_response()
    }
}
