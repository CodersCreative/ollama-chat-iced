use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct TtsQueryData {
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TtsResponse {
    pub spec: TtsResponseSpec,
    pub data: Vec<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TtsResponseSpec {
    pub sample_rate: u32,
    pub bits_per_sample: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TtsStreamResult {
    Idle,
    Err(String),
    Generating(TtsResponse),
    Finished,
}
