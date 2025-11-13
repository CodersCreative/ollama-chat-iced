pub mod stream;
use std::collections::HashMap;

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestFunctionMessageArgs,
        ChatCompletionRequestMessageContentPartImageArgs,
        ChatCompletionRequestMessageContentPartTextArgs, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, ChatCompletionRequestUserMessageContentPart,
        CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
    },
};
use axum::Json;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    CONN,
    errors::ServerError,
    files::get_file,
    messages::Role,
    providers::{PROVIDER_TABLE, Provider},
};

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct ChatQueryData {
    pub provider: String,
    pub model: String,
    pub messages: Vec<ChatQueryMessage>,
}

impl ChatQueryData {
    pub async fn get_chat_completion_request(
        &self,
    ) -> Result<CreateChatCompletionRequest, ServerError> {
        let mut messages = Vec::new();
        for chat in self.messages.iter() {
            messages.push(match chat.role {
                Role::User => {
                    let mut parts: Vec<ChatCompletionRequestUserMessageContentPart> = vec![
                        ChatCompletionRequestMessageContentPartTextArgs::default()
                            .text(chat.text.to_string())
                            .build()
                            .unwrap()
                            .into(),
                    ];

                    for file in chat.files.iter() {
                        match get_file(axum::extract::Path(file.clone()))
                            .await
                            .map(|x| x.0)
                        {
                            Ok(Some(image)) if image.file_type == crate::files::FileType::Image => {
                                parts.push(
                                    ChatCompletionRequestMessageContentPartImageArgs::default()
                                        .image_url(image.b64data)
                                        .build()
                                        .unwrap()
                                        .into(),
                                );
                            }
                            _ => {}
                        }
                    }

                    ChatCompletionRequestUserMessageArgs::default()
                        .content(parts)
                        .build()
                        .unwrap_or_default()
                        .into()
                }
                Role::AI => ChatCompletionRequestAssistantMessageArgs::default()
                    .content(chat.text.to_string())
                    .build()
                    .unwrap_or_default()
                    .into(),
                Role::System => ChatCompletionRequestSystemMessageArgs::default()
                    .content(chat.text.to_string())
                    .build()
                    .unwrap_or_default()
                    .into(),
                Role::Function => ChatCompletionRequestFunctionMessageArgs::default()
                    .content(chat.text.to_string())
                    .build()
                    .unwrap_or_default()
                    .into(),
            })
        }

        CreateChatCompletionRequestArgs::default()
            .model(self.model.clone())
            .messages(messages)
            .build()
            .map_err(|e| e.into())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatQueryMessage {
    pub text: String,
    pub files: Vec<String>,
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

#[axum::debug_handler]
pub async fn run(Json(data): Json<ChatQueryData>) -> Result<Json<ChatResponse>, ServerError> {
    let request = data.get_chat_completion_request().await?;

    let response = if let Some(provider) = CONN
        .select::<Option<Provider>>((PROVIDER_TABLE, &*data.provider))
        .await?
    {
        let provider = Into::<Client<OpenAIConfig>>::into(&provider);
        provider.chat().create(request).await?
    } else {
        panic!()
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
