use std::{thread, time::Duration};

use axum::Json;
use axum::response::IntoResponse;
use axum_streams::StreamBodyAs;
use futures::Stream;
use ochat_types::{chats::messages::Role, generation::text::ChatStreamResult};
use tokio_stream::StreamExt;

use crate::{
    CONN,
    generation::text::{
        ChatQueryData, ChatResponse, get_chat_completion_request, split_text_into_thinking,
    },
    providers::{PROVIDER_TABLE, provider_into_config},
};

async fn run_text_stream(data: ChatQueryData) -> impl Stream<Item = ChatStreamResult> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    let request = get_chat_completion_request(&data).await.unwrap();

    let mut response = if let Some(provider) = CONN
        .select((PROVIDER_TABLE, data.provider.trim()))
        .await
        .unwrap()
    {
        let provider = provider_into_config(&provider);
        provider.chat().create_stream(request).await.unwrap()
    } else {
        panic!()
    };

    tokio::spawn(async move {
        let mut content = String::new();
        while let Some(response) = response.next().await {
            match response {
                Ok(response) => {
                    let mut temp = String::new();
                    for choice in response.choices.iter() {
                        temp.push_str(&choice.delta.content.clone().unwrap_or_default());
                    }
                    content.push_str(&temp);

                    let _ = tx.send(ChatStreamResult::Generating(ChatResponse {
                        role: Role::AI,
                        content: temp,
                        thinking: None,
                        func_calls: Vec::new(),
                    }));
                }
                Err(e) => {
                    let _ = tx.send(ChatStreamResult::Err(e.to_string()));
                }
            }
        }

        let (content, thinking) = split_text_into_thinking(content.clone());

        let _ = tx.send(ChatStreamResult::Generated(ChatResponse {
            role: Role::AI,
            content,
            thinking,
            func_calls: Vec::new(),
        }));

        thread::sleep(Duration::from_millis(500));

        let _ = tx.send(ChatStreamResult::Finished);
    });

    return Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx));
}

#[axum::debug_handler]
pub async fn run(Json(data): Json<ChatQueryData>) -> impl IntoResponse {
    StreamBodyAs::json_nl(run_text_stream(data).await)
}
