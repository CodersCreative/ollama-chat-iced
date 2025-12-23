use crate::backend::errors::ServerError;
use axum::Json;
use natural_tts::{
    NaturalTts, NaturalTtsBuilder,
    models::{Spec, gtts::GttsModel, tts_rs::TtsModel},
};
use ochat_types::generation::tts::{TtsQueryData, TtsResponse, TtsResponseSpec};
use std::sync::{LazyLock, RwLock};
use text_splitter::TextSplitter;

pub mod stream;
static NATURAL_TTS: LazyLock<RwLock<NaturalTts>> = LazyLock::new(|| {
    RwLock::new(
        NaturalTtsBuilder::default()
            .default_model(natural_tts::Model::Gtts)
            .gtts_model(GttsModel::default())
            .tts_model(TtsModel::default())
            .build()
            .unwrap(),
    )
});

#[axum::debug_handler]
pub async fn run(Json(data): Json<TtsQueryData>) -> Result<Json<TtsResponse>, ServerError> {
    let texts = split_text_gtts(data.text);
    let mut data = Vec::new();
    let mut spec = None;

    for text in texts {
        let mut audio = NATURAL_TTS
            .write()
            .map_err(|e| ServerError::Unknown(e.to_string()))
            .unwrap()
            .synthesize_auto(text)
            .map_err(|e| ServerError::Unknown(e.to_string()))
            .unwrap();

        data.append(&mut audio.data);

        if spec.is_none() {
            let Spec::Wav(s) = audio.spec else { panic!() };

            spec = Some(TtsResponseSpec {
                sample_rate: s.sample_rate,
                bits_per_sample: s.bits_per_sample,
            });
        }
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

    return chunks
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
}
