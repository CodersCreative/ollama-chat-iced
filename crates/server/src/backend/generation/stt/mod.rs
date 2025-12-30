use std::path::PathBuf;

use crate::backend::{
    errors::ServerError,
    providers::hf::{pull::get_models_dir, stt::list_all_downloaded_models},
};
use axum::Json;
use ochat_types::{
    generation::stt::{SttQueryData, SttResponse},
    settings::SettingsProvider,
};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub async fn get_model_dir_and_name(data: &SettingsProvider) -> (PathBuf, String) {
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
        "stt".to_string(),
    )
    .await;

    (model_dir, name)
}

#[axum::debug_handler]
pub async fn run(Json(data): Json<SttQueryData>) -> Result<Json<SttResponse>, ServerError> {
    let model = if let Some(x) = data.model.clone() {
        x
    } else {
        let mut lst = list_all_downloaded_models().await?.0;
        if lst.is_empty() {
            return Err(ServerError::Unknown(String::from(
                "No STT models downloaded",
            )));
        }
        lst.remove(0)
    };

    let model_path = {
        let (dir, name) = get_model_dir_and_name(&model).await;
        dir.join(name)
    };

    let ctx = WhisperContext::new_with_params(
        &model_path.display().to_string(),
        WhisperContextParameters::default(),
    )?;

    let params = FullParams::new(SamplingStrategy::BeamSearch {
        beam_size: 5,
        patience: -1.0,
    });

    let mut state = ctx.create_state()?;
    state.full(params, &data.data[..])?;

    let mut text = String::new();

    for segment in state.as_iter() {
        text.push_str(segment.to_str()?);
    }

    Ok(Json(SttResponse { text }))
}
