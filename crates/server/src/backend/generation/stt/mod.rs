use crate::backend::errors::ServerError;
use axum::Json;
use ochat_types::generation::stt::{SttQueryData, SttResponse};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

#[axum::debug_handler]
pub async fn run(Json(data): Json<SttQueryData>) -> Result<Json<SttResponse>, ServerError> {
    let model_path = "";

    let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())?;

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
