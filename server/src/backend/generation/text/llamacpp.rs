use crate::backend::{
    errors::ServerError, files::get_file, generation::text::split_text_into_thinking,
    providers::hf::pull::get_models_dir,
};
use axum::{Json, extract::Path};
use futures::Stream;
use llama_cpp_2::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{AddBos, LlamaModel, Special, params::LlamaModelParams},
    sampling::LlamaSampler,
};
use ochat_types::generation::text::{ChatQueryData, ChatResponse, ChatStreamResult};
use ochat_types::{chats::messages::Role, files::FileType};
use std::{num::NonZeroU32, path::PathBuf};

const MAX_TOKENS: usize = 1048576;
const MAX_NEW_TOKENS: usize = 512;

pub async fn get_model_dir_and_name(data: &ChatQueryData) -> (PathBuf, String) {
    let (user, model, name) = {
        let provider = data.provider.trim().trim_start_matches("HF:");
        let (user, model) = provider.trim().split_once("/").unwrap();
        (
            user.to_string(),
            model.to_string(),
            data.model.trim().to_string(),
        )
    };
    let model_dir = get_models_dir(format!("{}/{}", user, model), "text".to_string()).await;

    (model_dir, name)
}

pub async fn run(data: ChatQueryData) -> Result<Json<ChatResponse>, ServerError> {
    let mut prompt = String::new();

    for chat in data.messages.iter() {
        let role_label = match chat.role {
            Role::User => "User",
            Role::AI => "Assistant",
            Role::System => "System",
            Role::Function => "Function",
        };

        prompt.push_str(&format!("{}: {}\n", role_label, chat.text));

        for file in chat.files.iter() {
            match get_file(Path(file.clone())).await.map(|x| x.0) {
                Ok(Some(image)) if image.file_type == FileType::Image => {
                    prompt.push_str(&format!(
                        "[image:data:{}/{};base64,{}]\n",
                        image.file_type,
                        image.filename.rsplit_once('.').unwrap().1,
                        image.b64data
                    ));
                }
                _ => {}
            }
        }
    }

    let model_path = {
        let (dir, name) = get_model_dir_and_name(&data).await;
        dir.join(name)
    };

    let prompt_clone = prompt.clone();

    let join = tokio::task::spawn_blocking(move || -> Result<String, String> {
        let backend = LlamaBackend::init().map_err(|e| e.to_string())?;

        let model_params = {
            #[cfg(any(feature = "cuda", feature = "vulkan", feature = "metal"))]
            {
                LlamaModelParams::default().with_n_gpu_layers(1000)
            }
            #[cfg(not(any(feature = "cuda", feature = "vulkan", feature = "metal")))]
            LlamaModelParams::default()
        };
        let model = LlamaModel::load_from_file(&backend, &model_path, &model_params)
            .map_err(|e| e.to_string())?;

        let ctx_params =
            LlamaContextParams::default().with_n_ctx(Some(NonZeroU32::new(2048).unwrap()));
        let mut ctx = model
            .new_context(&backend, ctx_params)
            .map_err(|e| e.to_string())?;

        let tokens_vec = model
            .str_to_token(&prompt_clone, AddBos::Always)
            .map_err(|e| e.to_string())?;

        let mut batch = LlamaBatch::new(MAX_TOKENS, 1);
        let last_index = (tokens_vec.len().saturating_sub(1)) as i32;

        for (i, token) in tokens_vec.iter().cloned().enumerate() {
            let i32i = i as i32;
            let is_last = i as i32 == last_index;
            batch
                .add(token, i32i, &[0], is_last)
                .map_err(|e| e.to_string())?;
        }

        ctx.decode(&mut batch).map_err(|e| e.to_string())?;

        // Initialize a chained sampler with repetition penalties and
        // temperature/top-k/top-p, ending with stochastic sampling.
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::penalties(64, 1.1, 0.0, 0.0),
            LlamaSampler::temp(0.8),
            LlamaSampler::top_k(40),
            LlamaSampler::top_p(0.95, 1),
            LlamaSampler::dist(1234),
        ]);

        // Tell sampler about the existing prompt tokens so it can account for
        // them when applying repetition penalties and other stateful samplers.
        sampler = sampler.with_tokens(tokens_vec.clone());

        let mut output = String::new();
        let mut n_cur = batch.n_tokens();

        let max_new_tokens = MAX_NEW_TOKENS;

        for _ in 0..max_new_tokens {
            let token = sampler.sample(&ctx, batch.n_tokens().saturating_sub(1) as i32);
            sampler.accept(token);

            if model.is_eog_token(token) {
                break;
            }

            let chunk = model
                .token_to_str(token, Special::Tokenize)
                .unwrap_or_default();

            output.push_str(&chunk);

            batch.clear();
            batch
                .add(token, n_cur as i32, &[0], true)
                .map_err(|e| e.to_string())?;

            ctx.decode(&mut batch).map_err(|e| e.to_string())?;

            n_cur += 1;
        }

        Ok(output)
    });

    let generated = join
        .await
        .map_err(|e| ServerError::Unknown(e.to_string()))?
        .map_err(|e| ServerError::Unknown(e))?;

    let (content, thinking) = split_text_into_thinking(generated);

    Ok(Json(ChatResponse {
        role: Role::AI,
        content,
        thinking,
        func_calls: Vec::new(),
    }))
}

pub async fn stream(data: ChatQueryData) -> impl Stream<Item = ChatStreamResult> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    let mut prompt = String::new();

    for chat in data.messages.iter() {
        let role_label = match chat.role {
            Role::User => "User",
            Role::AI => "Assistant",
            Role::System => "System",
            Role::Function => "Function",
        };

        prompt.push_str(&format!("{}: {}\n", role_label, chat.text));

        for file in chat.files.iter() {
            match get_file(Path(file.clone())).await.map(|x| x.0) {
                Ok(Some(image)) if image.file_type == FileType::Image => {
                    prompt.push_str(&format!(
                        "[image:data:{}/{};base64,{}]\n",
                        image.file_type,
                        image.filename.rsplit_once('.').unwrap().1,
                        image.b64data
                    ));
                }
                _ => {}
            }
        }
    }

    let model_path = {
        let (dir, name) = get_model_dir_and_name(&data).await;
        dir.join(name)
    };

    let prompt_clone = prompt.clone();
    let _ = tokio::task::spawn_blocking(move || {
        let backend = match LlamaBackend::init() {
            Ok(b) => b,
            Err(e) => {
                let _ = tx.send(ChatStreamResult::Err(e.to_string()));
                let _ = tx.send(ChatStreamResult::Finished);
                return;
            }
        };

        let model_params = {
            #[cfg(any(feature = "cuda", feature = "vulkan"))]
            {
                LlamaModelParams::default().with_n_gpu_layers(1000)
            }
            #[cfg(not(any(feature = "cuda", feature = "vulkan")))]
            LlamaModelParams::default()
        };

        let model = match LlamaModel::load_from_file(&backend, &model_path, &model_params) {
            Ok(m) => m,
            Err(e) => {
                let _ = tx.send(ChatStreamResult::Err(e.to_string()));
                let _ = tx.send(ChatStreamResult::Finished);
                return;
            }
        };

        let ctx_params = LlamaContextParams::default();
        let mut ctx = match model.new_context(&backend, ctx_params) {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(ChatStreamResult::Err(e.to_string()));
                let _ = tx.send(ChatStreamResult::Finished);
                return;
            }
        };

        let tokens_vec = match model.str_to_token(&prompt_clone, AddBos::Always) {
            Ok(t) => t,
            Err(e) => {
                let _ = tx.send(ChatStreamResult::Err(e.to_string()));
                let _ = tx.send(ChatStreamResult::Finished);
                return;
            }
        };

        let mut batch = LlamaBatch::new(MAX_TOKENS, 1);
        let last_index = (tokens_vec.len().saturating_sub(1)) as i32;
        for (i, token) in tokens_vec.iter().cloned().enumerate() {
            let i32i = i as i32;
            let is_last = i as i32 == last_index;
            if let Err(e) = batch.add(token, i32i, &[0], is_last) {
                let _ = tx.send(ChatStreamResult::Err(e.to_string()));
                let _ = tx.send(ChatStreamResult::Finished);
                return;
            }
        }

        if let Err(e) = ctx.decode(&mut batch) {
            let _ = tx.send(ChatStreamResult::Err(e.to_string()));
            let _ = tx.send(ChatStreamResult::Finished);
            return;
        }

        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::penalties(64, 1.1, 0.0, 0.0),
            LlamaSampler::temp(0.8),
            LlamaSampler::top_k(40),
            LlamaSampler::top_p(0.95, 1),
            LlamaSampler::dist(1234),
        ]);

        sampler = sampler.with_tokens(tokens_vec.clone());

        let mut n_cur = batch.n_tokens();

        let max_new_tokens = MAX_NEW_TOKENS;
        let mut generated = String::new();

        for _ in 0..max_new_tokens {
            let token = sampler.sample(&ctx, batch.n_tokens().saturating_sub(1) as i32);
            sampler.accept(token);

            if model.is_eog_token(token) {
                break;
            }

            let chunk = model
                .token_to_str(token, Special::Tokenize)
                .unwrap_or_default();

            generated.push_str(&chunk);

            let (content, thinking) = split_text_into_thinking(chunk);

            let _ = tx.send(ChatStreamResult::Generating(ChatResponse {
                role: Role::AI,
                content,
                thinking,
                func_calls: Vec::new(),
            }));

            batch.clear();
            if let Err(e) = batch.add(token, n_cur as i32, &[0], true) {
                let _ = tx.send(ChatStreamResult::Err(e.to_string()));
                break;
            }

            if let Err(e) = ctx.decode(&mut batch) {
                let _ = tx.send(ChatStreamResult::Err(e.to_string()));
                break;
            }

            n_cur += 1;
        }

        let (content, thinking) = split_text_into_thinking(generated);

        let _ = tx.send(ChatStreamResult::Generated(ChatResponse {
            role: Role::AI,
            content,
            thinking,
            func_calls: Vec::new(),
        }));

        let _ = tx.send(ChatStreamResult::Finished);
    });

    return Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx));
}
