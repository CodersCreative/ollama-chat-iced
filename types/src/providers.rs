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
    use crate::surreal::Datetime;
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
        pub base_model: Option<String>,
        pub architecture: Option<String>,
        #[serde(default = "Default::default")]
        pub parameters: u64,
        #[serde(default = "Default::default")]
        pub variants: HFModelVariants,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct HFModelVariant {
        pub model: String,
        pub name: String,
        pub size: Option<u64>,
    }

    impl HFModelVariant {
        pub fn variant(&self) -> Option<&str> {
            self.name
                .trim_end_matches(".gguf")
                .rsplit(['-', '.'])
                .next()
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
    }
}
