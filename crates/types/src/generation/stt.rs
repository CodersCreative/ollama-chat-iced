use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::generation::SoundSpec;

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct SttQueryData {
    pub spec: SoundSpec,
    pub data: Vec<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SttResponse {
    pub text: String,
}
