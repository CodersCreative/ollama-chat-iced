use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestFunctionMessageArgs,
    ChatCompletionRequestMessageContentPartImageArgs,
    ChatCompletionRequestMessageContentPartTextArgs, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, ChatCompletionRequestUserMessageContentPart,
    CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
};
use axum::Json;
use futures::Stream;
use ochat_types::{
    chats::messages::Role,
    files::FileType,
    generation::text::{ChatQueryData, ChatResponse, ChatStreamResult},
    options::GenOptionKey,
    providers::Provider,
};
use std::{thread, time::Duration};
use tokio_stream::StreamExt;

use crate::backend::{
    CONN,
    errors::ServerError,
    files::get_file,
    generation::text::split_text_into_thinking,
    options::relationships::get_default_gen_options_from_model,
    providers::{PROVIDER_TABLE, provider_into_config},
};

async fn get_chat_completion_request(
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
                            match ChatCompletionRequestMessageContentPartImageArgs::default()
                                .image_url(format!(
                                    "data:{}/{};base64,{}",
                                    image.file_type,
                                    image.filename.rsplit_once(".").unwrap().1,
                                    image.b64data
                                ))
                                .build()
                            {
                                Ok(x) => parts.push(x.into()),
                                Err(e) => eprintln!("{:?}", e),
                            }
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

    let mut a = CreateChatCompletionRequestArgs::default();
    let mut args = a.model(query.model.trim().to_string()).messages(messages);

    if let Ok(Json(Some(options))) = get_default_gen_options_from_model(axum::extract::Path((
        query.provider.clone(),
        query.model.clone(),
    )))
    .await
    {
        for option in options.data.iter().filter(|x| x.activated) {
            args = match option.key {
                GenOptionKey::TopP => args.top_p(option.value.as_f32()),
                GenOptionKey::Temperature => args.temperature(option.value.as_f32()),
                GenOptionKey::Seed => args.seed(option.value.as_i32()),
                GenOptionKey::RepeatPenalty => args.frequency_penalty(option.value.as_f32()),
                _ => continue,
            };
        }
    }

    args.build().map_err(|e| e.into())
}

pub async fn run(data: ChatQueryData) -> Result<Json<ChatResponse>, ServerError> {
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

pub async fn stream(data: ChatQueryData) -> impl Stream<Item = ChatStreamResult> {
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
                    eprintln!("{:?}", e);
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

        thread::sleep(Duration::from_millis(100));

        let _ = tx.send(ChatStreamResult::Finished);
    });

    return Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx));
}
