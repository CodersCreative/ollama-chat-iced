use crate::backend::{
    CONN,
    errors::ServerError,
    files::get_file,
    options::relationships::get_default_gen_options_from_model,
    providers::{PROVIDER_TABLE, generic_rig, provider_into_config},
    settings::get_settings,
    tools::builtin::{WebScraper, WebSearch},
};
use axum::Json;
use futures::{Stream, StreamExt};
use ochat_types::{
    chats::messages::Role,
    files::FileType,
    generation::text::{ChatQueryData, ChatResponse, ChatStreamResult, split_text_into_thinking},
    options::GenOptionKey,
    providers::Provider,
};
use rig::{
    OneOrMany,
    client::{CompletionClient, EmbeddingsClient},
    completion::{Completion, Prompt},
    embeddings::{EmbeddingsBuilder, ToolSchema},
    message::ImageMediaType,
    streaming::StreamingCompletion,
    tool::ToolSet,
    vector_store::in_memory_store::{InMemoryVectorIndex, InMemoryVectorStore},
};
use std::{thread, time::Duration};

type Agent = rig::agent::Agent<generic_rig::CompletionModel>;

async fn get_chat_completion_request(
    query: &ChatQueryData,
) -> Result<(Agent, Vec<rig::message::Message>), ServerError> {
    let Some(provider) = CONN
        .select::<Option<Provider>>((PROVIDER_TABLE, query.provider.trim()))
        .await?
    else {
        panic!()
    };

    let provider = provider_into_config(&provider);
    let mut agent = provider.agent(query.model.trim());
    let mut messages = Vec::new();
    for chat in query.messages.iter() {
        messages.push(match chat.role {
            Role::User => rig::message::Message::User {
                content: OneOrMany::many({
                    let mut parts = vec![rig::message::UserContent::text(chat.text.to_string())];
                    for file in chat.files.iter() {
                        match get_file(axum::extract::Path(file.clone()))
                            .await
                            .map(|x| x.0)
                        {
                            Ok(Some(image)) if image.file_type == FileType::Image => {
                                parts.push(rig::message::UserContent::image_base64(
                                    image.b64data,
                                    match image
                                        .filename
                                        .rsplit_once(".")
                                        .map(|x| x.1.trim())
                                        .unwrap()
                                    {
                                        "jpg" | "jpeg" => Some(ImageMediaType::JPEG),
                                        "gif" => Some(ImageMediaType::GIF),
                                        "png" => Some(ImageMediaType::PNG),
                                        "webp" => Some(ImageMediaType::WEBP),
                                        "heic" => Some(ImageMediaType::HEIC),
                                        "heif" => Some(ImageMediaType::HEIF),
                                        "svg" => Some(ImageMediaType::SVG),
                                        _ => None,
                                    },
                                    None,
                                ))
                            }
                            Ok(Some(file)) => parts.push(rig::message::UserContent::Document(
                                rig::message::Document {
                                    additional_params: None,
                                    data: rig::message::DocumentSourceKind::Base64(file.b64data),
                                    media_type: Some(rig::message::DocumentMediaType::MARKDOWN),
                                },
                            )),
                            _ => {}
                        }
                    }
                    parts
                })
                .unwrap(),
            },
            Role::AI => rig::message::Message::Assistant {
                id: None,
                content: OneOrMany::one(rig::message::AssistantContent::text(
                    chat.text.to_string(),
                )),
            },
            Role::System => {
                agent = agent.append_preamble(&chat.text.to_string());
                continue;
            }
            Role::Function => todo!(),
        })
    }

    if let Ok(Json(Some(options))) = get_default_gen_options_from_model(axum::extract::Path((
        query.provider.clone(),
        query.model.clone(),
    )))
    .await
    {
        for option in options.data {
            match option.key {
                GenOptionKey::Temperature => {
                    agent = agent.temperature(option.value.as_f32() as f64)
                }
                _ => {}
            }
        }
    }

    Ok((
        if query.force_disable_tools || query.tools.is_empty() {
            agent.build()
        } else {
            match get_tools().await {
                Ok(Some(tools))
                    if check_tool_compatibality(&provider, query.model.trim()).await =>
                {
                    agent.dynamic_tools(tools.0, tools.1, tools.2).build()
                }
                _ => agent.build(),
            }
        },
        messages,
    ))
}

pub async fn check_tool_compatibality(client: &generic_rig::Client, model: &str) -> bool {
    let agent = client.agent(model.trim());

    match agent.tool(WebScraper).build().prompt("Hello").await {
        Ok(_) => true,
        _ => false,
    }
}

pub async fn get_tools() -> Result<
    Option<(
        usize,
        InMemoryVectorIndex<generic_rig::EmbeddingModel<reqwest::Client>, ToolSchema>,
        ToolSet,
    )>,
    ServerError,
> {
    let toolset = ToolSet::builder()
        .dynamic_tool(WebScraper)
        .dynamic_tool(WebSearch)
        .build();

    let Some(provider) = get_settings().await?.0.embeddings_provider else {
        return Ok(None);
    };
    let model = provider.model;

    let Some(provider) = CONN
        .select::<Option<Provider>>((PROVIDER_TABLE, provider.provider.trim()))
        .await?
    else {
        panic!()
    };

    let client = provider_into_config(&provider);

    let embedding_model = client.embedding_model(model);
    let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
        .documents(toolset.schemas()?)?
        .build()
        .await?;

    let vector_store =
        InMemoryVectorStore::from_documents_with_id_f(embeddings, |tool| tool.name.clone());
    let index = vector_store.index(embedding_model);

    Ok(Some((2, index, toolset)))
}

pub async fn run(data: ChatQueryData) -> Result<Json<ChatResponse>, ServerError> {
    let request = get_chat_completion_request(&data).await?;

    let response = {
        let mut messages = request.1;
        request
            .0
            .completion(
                if messages.len() % 2 == 0 {
                    rig::message::Message::user("Now generate from your previous instructions...")
                } else {
                    messages.pop().unwrap()
                },
                messages,
            )
            .await?
            .send()
            .await?
    };

    let mut content = String::new();
    let mut thinking = String::new();

    for choice in response.choice.iter() {
        match choice {
            rig::message::AssistantContent::Text(x) => content.push_str(&x.text),
            rig::message::AssistantContent::Reasoning(x) => {
                for x in x.reasoning.iter() {
                    thinking.push_str(&x)
                }
            }
            _ => {}
        }
    }

    let (content, thinking2) = split_text_into_thinking(content);

    Ok(Json(ChatResponse {
        role: Role::AI,
        content,
        thinking: if thinking.is_empty() {
            thinking2
        } else {
            if let Some(thinking2) = thinking2 {
                thinking.push_str(&thinking2);
            }
            Some(thinking)
        },
        func_calls: Vec::new(),
    }))
}

pub async fn stream(data: ChatQueryData) -> impl Stream<Item = ChatStreamResult> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        let request = match get_chat_completion_request(&data).await {
            Ok(x) => x,
            Err(e) => {
                let _ = tx.send(ChatStreamResult::Err(e.to_string()));
                let _ = tx.send(ChatStreamResult::Finished);
                return;
            }
        };

        let mut response = {
            let mut messages = request.1;
            match request
                .0
                .stream_completion(
                    if messages.len() % 2 == 0 {
                        rig::message::Message::user(
                            "Now generate from your previous instructions...",
                        )
                    } else {
                        messages.pop().unwrap()
                    },
                    messages,
                )
                .await
            {
                Ok(x) => match x.stream().await {
                    Ok(x) => x,
                    Err(e) => {
                        let _ = tx.send(ChatStreamResult::Err(e.to_string()));
                        let _ = tx.send(ChatStreamResult::Finished);
                        return;
                    }
                },
                Err(e) => {
                    let _ = tx.send(ChatStreamResult::Err(e.to_string()));
                    let _ = tx.send(ChatStreamResult::Finished);
                    return;
                }
            }
        };

        let mut content = String::new();
        let mut thinking = String::new();
        while let Some(response) = response.next().await {
            match response {
                Ok(response) => {
                    let mut temp = String::new();
                    let mut temp_thinking = String::new();
                    match response {
                        rig::streaming::StreamedAssistantContent::Text(x) => temp.push_str(&x.text),
                        rig::streaming::StreamedAssistantContent::Reasoning(x) => {
                            for x in x.reasoning.iter() {
                                temp_thinking.push_str(x)
                            }
                        }
                        rig::streaming::StreamedAssistantContent::ReasoningDelta {
                            id: _,
                            reasoning,
                        } => temp_thinking.push_str(&reasoning),
                        _ => {}
                    }
                    content.push_str(&temp);
                    thinking.push_str(&temp_thinking);

                    let _ = tx.send(ChatStreamResult::Generating(ChatResponse {
                        role: Role::AI,
                        content: temp,
                        thinking: if temp_thinking.is_empty() {
                            None
                        } else {
                            Some(temp_thinking)
                        },
                        func_calls: Vec::new(),
                    }));
                }
                Err(e) => {
                    let _ = tx.send(ChatStreamResult::Err(e.to_string()));
                }
            }
        }
        let (content, thinking2) = split_text_into_thinking(content);
        let _ = tx.send(ChatStreamResult::Generated(ChatResponse {
            role: Role::AI,
            content,
            thinking: if thinking.is_empty() {
                thinking2
            } else {
                if let Some(thinking2) = thinking2 {
                    thinking.push_str(&thinking2);
                }
                Some(thinking)
            },
            func_calls: Vec::new(),
        }));

        thread::sleep(Duration::from_millis(20));

        let _ = tx.send(ChatStreamResult::Finished);
    });

    Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx))
}
