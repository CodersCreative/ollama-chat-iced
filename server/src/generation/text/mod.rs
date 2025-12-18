use axum::{Json, response::IntoResponse};
use axum_streams::StreamBodyAs;
use ochat_types::generation::text::{ChatQueryData, ChatResponse};

use crate::errors::ServerError;

pub mod api;
pub mod llamacpp;

#[axum::debug_handler]
pub async fn run(Json(data): Json<ChatQueryData>) -> Result<Json<ChatResponse>, ServerError> {
    if data.provider.starts_with("HF:") {
        llamacpp::run(data).await
    } else {
        api::run(data).await
    }
}

#[axum::debug_handler]
pub async fn stream(Json(data): Json<ChatQueryData>) -> impl IntoResponse {
    if data.provider.starts_with("HF:") {
        StreamBodyAs::json_nl(llamacpp::stream(data).await)
    } else {
        StreamBodyAs::json_nl(api::stream(data).await)
    }
}

pub fn split_text_into_thinking(text: String) -> (String, Option<String>) {
    let text = text
        .trim()
        .trim_start_matches("AI")
        .trim_start_matches(":")
        .trim()
        .to_string();
    let deal_with_end = |text: String| -> (String, Option<String>) {
        if text.contains("</think>") {
            let split = text.rsplit_once("</think>").unwrap();

            (
                split.1.trim().to_string(),
                if !split.0.trim().is_empty() {
                    Some(split.0.trim().to_string())
                } else {
                    None
                },
            )
        } else {
            (text.trim().to_string(), None)
        }
    };

    if text.contains("<think>") {
        let c = text.clone();
        let split = c.split_once("<think>").unwrap();
        let mut content = split.0.to_string();
        let temp = deal_with_end(split.1.trim().to_string());
        content.push_str(&temp.0);

        (content.trim().to_string(), temp.1)
    } else {
        deal_with_end(text)
    }
}
