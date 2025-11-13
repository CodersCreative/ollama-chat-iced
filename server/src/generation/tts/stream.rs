use axum::{Json, response::IntoResponse};
use axum_streams::StreamBodyAs;
use futures::Stream;
use serde::{Deserialize, Serialize};

use crate::{
    errors::ServerError,
    generation::tts::{NATURAL_TTS, TtsQueryData, TtsResponse, TtsResponseSpec, split_text_gtts},
};

use natural_tts::models::Spec;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TtsStreamResult {
    Err(String),
    Generating(TtsResponse),
    Finished,
}

async fn run_tts_stream(data: TtsQueryData) -> impl Stream<Item = TtsStreamResult> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        let texts = split_text_gtts(data.text);

        for text in texts {
            let audio = NATURAL_TTS
                .write()
                .map_err(|e| ServerError::Unknown(e.to_string()))
                .unwrap()
                .synthesize_auto(text)
                .map_err(|e| ServerError::Unknown(e.to_string()))
                .unwrap();

            let data = audio.data;
            let Spec::Wav(spec) = audio.spec else {
                panic!()
            };

            let _ = tx.send(TtsStreamResult::Generating(TtsResponse {
                spec: TtsResponseSpec {
                    sample_rate: spec.sample_rate,
                    bits_per_sample: spec.bits_per_sample,
                },
                data,
            }));
        }
        let _ = tx.send(TtsStreamResult::Finished);
    });

    return Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx));
}

#[axum::debug_handler]
pub async fn run(Json(data): Json<TtsQueryData>) -> impl IntoResponse {
    StreamBodyAs::json_nl(run_tts_stream(data).await)
}
