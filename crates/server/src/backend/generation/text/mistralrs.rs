use crate::backend::{
    errors::ServerError, generation::text::split_text_into_thinking,
    providers::hf::pull::get_models_dir,
};
use axum::Json;
use futures::Stream;
use mistralrs::{
    DefaultSchedulerMethod, Model, ModelDType, ResponseOk, SchedulerConfig, VisionMessages,
    best_device,
    core::{
        AdapterPaths, AutoLoaderBuilder, EmbeddingSpecificConfig, LocalModelPaths,
        MistralRsBuilder, ModelPaths, NormalSpecificConfig, VisionSpecificConfig,
    },
};

use ochat_types::{
    chats::messages::Role,
    generation::text::{ChatQueryData, ChatQueryMessage, ChatResponse, ChatStreamResult},
};
use std::{fs, path::PathBuf, thread, time::Duration};

pub async fn get_model_dir_and_name(data: &ChatQueryData) -> (PathBuf, String) {
    let (user, model, name) = {
        let provider = data.provider.trim().split_once(":").unwrap().1;
        let (user, model) = provider.trim().split_once("/").unwrap();
        (
            user.to_string(),
            model.to_string(),
            data.model.trim().to_string(),
        )
    };
    let model_dir = get_models_dir(
        name.clone(),
        format!("{}/{}", user, model),
        "text".to_string(),
    )
    .await;

    (model_dir, name)
}

pub async fn get_model(data: &ChatQueryData) -> Result<Model, ServerError> {
    let (path, name) = get_model_dir_and_name(data).await;

    let get_file_if_exists = |path: PathBuf| -> Option<PathBuf> {
        if fs::exists(&path).unwrap_or_default() {
            Some(path)
        } else {
            None
        }
    };

    let loader = AutoLoaderBuilder::new(
        NormalSpecificConfig::default(),
        VisionSpecificConfig::default(),
        EmbeddingSpecificConfig::default(),
        None,
        get_file_if_exists(path.join("tokenizer.json")).map(|x| x.to_str().unwrap().to_string()),
        data.provider.trim().split_once(":").unwrap().1.to_string(),
        true,
        None,
    )
    .build();

    let paths: Box<dyn ModelPaths> = Box::new(LocalModelPaths {
        tokenizer_filename: path.join("tokenizers.json"),
        config_filename: path.join("config.json"),
        template_filename: None,
        filenames: vec![path.join(name)],
        adapter_paths: AdapterPaths::None,
        gen_conf: get_file_if_exists(path.join("generation_config.json")),
        preprocessor_config: get_file_if_exists(path.join("preprocessor_config.json")),
        processor_config: get_file_if_exists(path.join("processor_config.json")),
        chat_template_json_filename: None,
    });

    let pipeline = loader
        .load_model_from_path(
            &paths,
            &ModelDType::Auto,
            &best_device(false).map_err(|e| ServerError::Unknown(e.to_string()))?,
            false,
            mistralrs::DeviceMapSetting::Auto(mistralrs::AutoDeviceMapParams::default_text()),
            None,
            None,
        )
        .map_err(|e| ServerError::Unknown(e.to_string()))?;

    let mut runner = MistralRsBuilder::new(
        pipeline,
        SchedulerConfig::DefaultScheduler {
            method: DefaultSchedulerMethod::Fixed(32.try_into().unwrap()),
        },
        false,
        None,
    );

    runner = runner.with_prefix_cache_n(16);
    Ok(Model::new(runner.build().await))
}

pub fn get_messages_from_chat_query(
    messages: Vec<ChatQueryMessage>,
) -> Result<VisionMessages, String> {
    let mut msgs = VisionMessages::new();
    msgs = msgs.enable_thinking(true);

    for message in messages {
        msgs = msgs.add_message(
            match message.role {
                Role::User => mistralrs::TextMessageRole::User,
                Role::Function => mistralrs::TextMessageRole::Tool,
                Role::AI => mistralrs::TextMessageRole::Assistant,
                Role::System => mistralrs::TextMessageRole::System,
            },
            message.text,
        );
    }

    Ok(msgs)
}

pub async fn run(data: ChatQueryData) -> Result<Json<ChatResponse>, ServerError> {
    let model = get_model(&data).await?;

    let response = model
        .send_chat_request(
            get_messages_from_chat_query(data.messages).map_err(ServerError::Unknown)?,
        )
        .await
        .map_err(|e| ServerError::Unknown(e.to_string()))?;

    let mut content = String::new();
    let mut thinking = String::new();

    for choice in response.choices.iter() {
        content.push_str(&choice.message.content.clone().unwrap_or_default());
        thinking.push_str(&choice.message.reasoning_content.clone().unwrap_or_default());
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
        let model = get_model(&data).await.unwrap();
        let mut response = match model
            .stream_chat_request(get_messages_from_chat_query(data.messages).unwrap())
            .await
        {
            Ok(x) => x,
            Err(e) => {
                let _ = tx.send(ChatStreamResult::Err(e.to_string()));
                let _ = tx.send(ChatStreamResult::Finished);
                return;
            }
        };
        let mut content = String::new();
        let mut thinking = String::new();
        while let Some(response) = response.next().await {
            match response.as_result() {
                Ok(response) => {
                    let mut temp = String::new();
                    let mut temp_thinking = String::new();

                    match response {
                        ResponseOk::Done(response) => {
                            for choice in response.choices.iter() {
                                temp.push_str(&choice.message.content.clone().unwrap_or_default());
                                temp_thinking.push_str(
                                    &choice.message.reasoning_content.clone().unwrap_or_default(),
                                );
                            }
                        }
                        ResponseOk::Chunk(response) => {
                            for choice in response.choices.iter() {
                                temp.push_str(&choice.delta.content.clone().unwrap_or_default());
                                temp_thinking.push_str(
                                    &choice.delta.reasoning_content.clone().unwrap_or_default(),
                                );
                            }
                        }
                        ResponseOk::CompletionChunk(response) => {
                            for choice in response.choices.iter() {
                                temp.push_str(&choice.text);
                            }
                        }
                        ResponseOk::CompletionDone(response) => {
                            for choice in response.choices.iter() {
                                temp.push_str(&choice.text);
                            }
                        }
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
