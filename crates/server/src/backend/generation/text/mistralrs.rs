use crate::backend::{
    errors::ServerError, generation::text::split_text_into_thinking,
    providers::hf::pull::get_models_dir,
};
use axum::Json;
use futures::Stream;
use mistralrs::{
    DefaultSchedulerMethod, GgufModelBuilder, Model, ModelDType, ResponseOk, SchedulerConfig,
    VisionMessages, best_device,
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

    let find_first_existing = |candidates: &[&str]| -> Option<PathBuf> {
        for candidate in candidates {
            if let Some(found) = get_file_if_exists(path.join(candidate)) {
                return Some(found);
            }
        }
        None
    };

    let tokenizer_path = find_first_existing(&[
        "tokenizer.json",
        "tokenizer.model",
        "tokenizer.json.sec",
        "tokenizer.model.sec",
        "tokenizer.json.ter",
        "tokenizer.model.ter",
    ])
    .ok_or_else(|| ServerError::Unknown("Missing tokenizer file in model dir".to_string()))?;

    let chat_template_path = find_first_existing(&[
        "chat_template.jinja",
        "chat_template.jinja.sec",
        "chat_template.jinja.ter",
    ]);

    let tokenizer_config_path = find_first_existing(&[
        "tokenizer_config.json",
        "tokenizer_config.json.sec",
        "tokenizer_config.json.ter",
    ]);

    let config_path =
        find_first_existing(&["config.json", "config.json.sec", "config.json.ter"])
            .ok_or_else(|| ServerError::Unknown("Missing config.json in model dir".to_string()))?;

    let mut weight_files: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(&path) {
        for entry in entries.flatten() {
            let file_path = entry.path();
            if let Some(ext) = file_path.extension().and_then(|x| x.to_str()) {
                let is_weight = matches!(ext, "safetensors" | "bin" | "gguf");
                if is_weight {
                    weight_files.push(file_path);
                }
            }
        }
    }

    let explicit = get_file_if_exists(path.join(&name));
    if let Some(explicit) = explicit {
        if !weight_files.contains(&explicit) {
            weight_files.push(explicit);
        }
    }

    if weight_files.is_empty() {
        return Err(ServerError::Unknown(
            "No model weight files (.safetensors/.bin/.gguf) found".to_string(),
        ));
    }

    weight_files.sort();

    let loader = AutoLoaderBuilder::new(
        NormalSpecificConfig::default(),
        VisionSpecificConfig::default(),
        EmbeddingSpecificConfig::default(),
        None,
        if tokenizer_path.extension().and_then(|x| x.to_str()) == Some("json") {
            Some(tokenizer_path.to_string_lossy().to_string())
        } else {
            None
        },
        data.provider.trim().split_once(":").unwrap().1.to_string(),
        true,
        None,
    )
    .build();

    let paths: Box<dyn ModelPaths> = Box::new(LocalModelPaths {
        tokenizer_filename: tokenizer_path.clone(),
        config_filename: config_path,
        template_filename: chat_template_path.clone(),
        filenames: weight_files.clone(),
        adapter_paths: AdapterPaths::None,
        gen_conf: get_file_if_exists(path.join("generation_config.json")),
        preprocessor_config: get_file_if_exists(path.join("preprocessor_config.json")),
        processor_config: get_file_if_exists(path.join("processor_config.json")),
        chat_template_json_filename: tokenizer_config_path,
    });

    let gguf_files: Vec<String> = weight_files
        .iter()
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("gguf"))
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    if !gguf_files.is_empty() {
        let mut builder = GgufModelBuilder::new(
            data.provider.trim().split_once(":").unwrap().1.to_string(),
            gguf_files,
        )
        .with_device_mapping(mistralrs::DeviceMapSetting::Auto(
            mistralrs::AutoDeviceMapParams::default_text(),
        ));

        if let Some(template) = &chat_template_path {
            builder = builder.with_chat_template(template.to_string_lossy().to_string());
        }

        if tokenizer_path.extension().and_then(|x| x.to_str()) == Some("json") {
            builder = builder.with_tokenizer_json(tokenizer_path.to_string_lossy().to_string());
        }

        return Ok(builder
            .build()
            .await
            .map_err(|e| ServerError::Unknown(e.to_string()))?);
    }

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
        let model = match get_model(&data).await {
            Ok(m) => m,
            Err(e) => {
                let _ = tx.send(ChatStreamResult::Err(e.to_string()));
                let _ = tx.send(ChatStreamResult::Finished);
                return;
            }
        };

        let msgs = match get_messages_from_chat_query(data.messages) {
            Ok(m) => m,
            Err(e) => {
                let _ = tx.send(ChatStreamResult::Err(e));
                let _ = tx.send(ChatStreamResult::Finished);
                return;
            }
        };

        let mut response = match model.stream_chat_request(msgs).await {
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
