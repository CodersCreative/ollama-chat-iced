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
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self.to_string())).into_response()
    }
}

impl From<surrealdb::Error> for ServerError {
    fn from(error: surrealdb::Error) -> Self {
        eprintln!("{error}");
        Self::Surreal(error)
    }
}
