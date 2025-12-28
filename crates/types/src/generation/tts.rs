use crate::generation::SoundSpec;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct TtsQueryData {
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TtsResponse {
    pub spec: SoundSpec,
    pub data: Vec<f32>,
}
