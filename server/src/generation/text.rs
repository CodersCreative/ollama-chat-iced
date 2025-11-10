use std::collections::HashMap;

use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestFunctionMessageArgs,
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{chats::Role, errors::ServerError, providers::PROVIDERS};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatQueryData {
    pub provider: String,
    pub model: String,
    pub messages: Vec<ChatQueryMessage>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatQueryMessage {
    pub text: String,
    pub role: Role,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FunctionCall {
    pub name: String,
    pub result: Option<Value>,
    pub args: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatResponse {
    pub role: Role,
    pub content: String,
    pub thinking: Option<String>,
    pub func_calls: Vec<FunctionCall>,
}

pub async fn run(Json(data): Json<ChatQueryData>) -> Result<Json<ChatResponse>, ServerError> {
    let request = CreateChatCompletionRequestArgs::default()
        .model(data.model)
        .messages(
            data.messages
                .into_iter()
                .map(|chat| match chat.role {
                    Role::User => ChatCompletionRequestUserMessageArgs::default()
                        .content(chat.text)
                        .build()
                        .unwrap_or_default()
                        .into(),
                    Role::AI => ChatCompletionRequestAssistantMessageArgs::default()
                        .content(chat.text)
                        .build()
                        .unwrap_or_default()
                        .into(),
                    Role::System => ChatCompletionRequestSystemMessageArgs::default()
                        .content(chat.text)
                        .build()
                        .unwrap_or_default()
                        .into(),
                    Role::Function => ChatCompletionRequestFunctionMessageArgs::default()
                        .content(chat.text)
                        .build()
                        .unwrap_or_default()
                        .into(),
                })
                .collect::<Vec<ChatCompletionRequestMessage>>(),
        )
        .build()
        .unwrap();

    let response = {
        let providers = PROVIDERS.read().unwrap();
        let provider = providers.get(&data.provider);
        provider.unwrap().client.chat().create(request).await?
    };

    let mut value = String::new();

    for choice in response.choices.iter() {
        value.push_str(&choice.message.content.clone().unwrap_or_default());
    }

    let (content, thinking) = split_text_into_thinking(value);

    Ok(Json(ChatResponse {
        role: Role::AI,
        content,
        thinking,
        func_calls: Vec::new(),
    }))
}
pub fn split_text_into_thinking(text: String) -> (String, Option<String>) {
    if text.contains("<think>") {
        let c = text.clone();
        let split = c.split_once("<think>").unwrap();
        let mut content = split.0.to_string();
        let think = if split.1.contains("</think>") {
            let split2 = split.1.rsplit_once("</think>").unwrap();
            content.push_str(split2.1);
            split2.0.to_string()
        } else {
            split.1.to_string()
        };

        (
            content.trim().to_string(),
            if !think.trim().is_empty() {
                Some(think.trim().to_string())
            } else {
                None
            },
        )
    } else {
        (text, None)
    }
}
