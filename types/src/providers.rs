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
    pub enum PullModelStreamResult {
        Idle,
        Err(String),
        Pulling(PullModelResponse),
        Finished,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, Default)]
    pub struct PullModelResponse {
        pub status: String,
        pub digest: Option<String>,
        pub total: Option<u64>,
        pub completed: Option<u64>,
    }
}
