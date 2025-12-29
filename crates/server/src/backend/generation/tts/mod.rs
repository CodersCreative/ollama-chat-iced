use crate::backend::errors::ServerError;
use axum::Json;
use natural_tts::{
    NaturalTts, NaturalTtsBuilder,
    models::{Spec, gtts::GttsModel, parler::ParlerModel},
};
use ochat_types::generation::{
    SoundSpec,
    tts::{TtsQueryData, TtsResponse},
};
use std::{
    path::PathBuf,
    sync::{LazyLock, RwLock},
};
use text_splitter::TextSplitter;

static NATURAL_TTS: LazyLock<RwLock<NaturalTts>> = LazyLock::new(|| {
    RwLock::new(
        NaturalTtsBuilder::default()
            .default_model(natural_tts::Model::Parler)
            .gtts_model(GttsModel::default())
            .parler_model(ParlerModel::default())
            .build()
            .unwrap(),
    )
});

#[axum::debug_handler]
pub async fn run(Json(data): Json<TtsQueryData>) -> Result<Json<TtsResponse>, ServerError> {
    let text = data.text;
    let mut data = Vec::new();
    let mut spec = None;

    let mut audio = NATURAL_TTS
        .write()
        .map_err(|e| ServerError::Unknown(e.to_string()))?
        .synthesize(text, &PathBuf::from("output.wav"))
        .map_err(|e| ServerError::Unknown(e.to_string()))?;

    data.append(&mut audio.data);

    if spec.is_none() {
        let Spec::Wav(s) = audio.spec else { panic!() };

        spec = Some(SoundSpec {
            sample_rate: s.sample_rate,
        });
    }

    Ok(Json(TtsResponse {
        spec: spec.unwrap(),
        data,
    }))
}

pub fn split_text_gtts(text: String) -> Vec<String> {
    split_text_with_len(100, text)
}

pub fn split_text_with_len(len: usize, text: String) -> Vec<String> {
    let splitter = TextSplitter::new(len);

    let chunks = splitter.chunks(&text).collect::<Vec<&str>>();

    chunks
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
}
