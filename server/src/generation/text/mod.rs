pub mod stream;

use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestFunctionMessageArgs,
    ChatCompletionRequestMessageContentPartImageArgs,
    ChatCompletionRequestMessageContentPartTextArgs, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, ChatCompletionRequestUserMessageContentPart,
    CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
};
use axum::Json;
use ochat_types::{
    chats::messages::Role,
    files::FileType,
    generation::text::{ChatQueryData, ChatResponse},
    providers::Provider,
};

use crate::{
    CONN,
    errors::ServerError,
    files::get_file,
    providers::{PROVIDER_TABLE, provider_into_config},
};

pub async fn get_chat_completion_request(
    query: &ChatQueryData,
) -> Result<CreateChatCompletionRequest, ServerError> {
    let mut messages = Vec::new();
    for chat in query.messages.iter() {
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
                        Ok(Some(image)) if image.file_type == FileType::Image => {
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
        .model(query.model.trim().to_string())
        .messages(messages)
        .build()
        .map_err(|e| e.into())
}

#[axum::debug_handler]
pub async fn run(Json(data): Json<ChatQueryData>) -> Result<Json<ChatResponse>, ServerError> {
    let request = get_chat_completion_request(&data).await?;

    let response = if let Some(provider) = CONN
        .select::<Option<Provider>>((PROVIDER_TABLE, data.provider.trim()))
        .await?
    {
        let provider = provider_into_config(&provider);
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
