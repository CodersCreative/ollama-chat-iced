use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{generation::SoundSpec, settings::SettingsProvider};

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct SttQueryData {
    #[builder(default = "None")]
    pub model: Option<SettingsProvider>,
    pub spec: SoundSpec,
    pub data: Vec<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SttResponse {
    pub text: String,
}
