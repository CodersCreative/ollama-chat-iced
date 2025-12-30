use std::fmt::Display;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::surreal::RecordId;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default, Eq, PartialOrd, Ord)]
pub enum ProviderType {
    OpenAI,
    Gemini,
    #[default]
    Ollama,
}

impl Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenAI => write!(f, "Open AI"),
            Self::Gemini => write!(f, "Gemini"),
            Self::Ollama => write!(f, "Ollama"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct ProviderData {
    pub name: String,
    pub url: String,
    pub api_key: String,
    #[builder(default = "ProviderType::Ollama")]
    pub provider_type: ProviderType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Provider {
    pub name: String,
    pub url: String,
    pub api_key: String,
    pub provider_type: ProviderType,
    pub id: RecordId,
}

impl Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Model {
    pub id: String,
    pub object: Option<String>,
    pub created: Option<u32>,
    pub owned_by: Option<String>,
}

pub mod ollama {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct OllamaModelsInfo {
        #[serde(default = "String::new")]
        pub name: String,
        pub url: String,
        pub tags: Vec<Vec<String>>,
        pub author: String,
        pub categories: Vec<String>,
        pub languages: Vec<String>,
        pub description: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub enum OllamaPullModelStreamResult {
        Idle,
        Err(String),
        Pulling(OllamaPullModelResponse),
        Finished,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, Default)]
    pub struct OllamaPullModelResponse {
        pub status: String,
        pub digest: Option<String>,
        pub total: Option<u64>,
        pub completed: Option<u64>,
    }
}

pub mod hf {
    use serde_json::Value;

    use crate::{settings::parse_provider_name, surreal::Datetime};
    use std::collections::HashMap;

    use super::*;

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct DownloadedHFModels {
        pub variants: Vec<HFModelVariant>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct HFModel {
        pub id: String,
        #[serde(alias = "lastModified")]
        pub last_modified: Datetime,
        pub downloads: u64,
        pub likes: u64,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct HFModelDetails {
        #[serde(default = "Default::default")]
        pub id: String,
        #[serde(default = "Default::default")]
        pub description: String,
        #[serde(alias = "lastModified")]
        pub last_modified: Datetime,
        pub downloads: u64,
        pub likes: u64,
        base_model: Option<String>,
        pub pipeline_tag: Option<String>,
        pub architecture: Option<String>,
        #[serde(alias = "cardData")]
        #[serde(default = "Default::default")]
        pub card_data: CardData,
        #[serde(default = "Default::default")]
        pub parameters: u64,
        #[serde(default = "Default::default")]
        pub variants: HFModelVariants,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, Default)]
    pub struct CardData {
        #[serde(default = "Default::default")]
        base_model: Option<Value>,
    }

    impl HFModelDetails {
        pub fn get_base_model(&self) -> Option<String> {
            if let Some(x) = self.base_model.clone() {
                Some(x)
            } else if let Some(x) = self.card_data.base_model.clone() {
                if x.is_array() {
                    let lst = x.as_array().unwrap();
                    lst.first().map(|x| x.as_str().unwrap().to_string())
                } else {
                    Some(x.as_str().unwrap().to_string())
                }
            } else {
                None
            }
        }
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct HFModelVariant {
        pub model: String,
        pub name: String,
        pub model_type: ModelType,
        pub size: Option<u64>,
        pub is_sharded: bool,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum ModelType {
        Text,
        Stt,
        Tts,
    }

    impl HFModelVariant {
        pub fn variant(&self) -> Option<String> {
            Some(parse_provider_name(&self.name, &self.model))
        }
    }

    #[derive(Serialize, Deserialize, Clone, Debug, Default)]
    pub struct HFModelVariants(pub HashMap<u64, Vec<HFModelVariant>>);

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub enum HFPullModelStreamResult {
        Idle,
        Err(String),
        Pulling(HFPullModelResponse),
        Finished,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, Default)]
    pub struct HFPullModelResponse {
        pub total: Option<u64>,
        pub completed: Option<u64>,
        pub speed: Option<f64>,
    }
}
