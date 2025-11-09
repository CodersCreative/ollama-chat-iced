use async_openai::{config::OpenAIConfig, Client};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::{collections::HashMap, io::Read};
use tokio_stream::{Stream, StreamExt};

use crate::{common::Id, utils::get_path_settings};

pub const PROVIDERS_FILE: &str = "providers.json";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SavedProviders(pub HashMap<Id, SavedProvider>);

impl Default for SavedProviders {
    fn default() -> Self {
        let mut map = HashMap::new();
        if let Some(x) = get_local_ollama() {
            map.insert(Id::new(), x);
        }
        Self(map)
    }
}
fn get_local_ollama() -> Option<SavedProvider> {
    let url = "http://localhost:11434/v1".to_string();
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    if tokio_runtime
        .block_on(reqwest::Client::new().head(&url).send())
        .is_ok()
    {
        Some(SavedProvider {
            name: String::from("Local Ollama"),
            url,
            api_key: String::from("ollama"),
            provider_type: ProviderType::Ollama,
        })
    } else {
        None
    }
}

#[derive(Debug, Clone)]
pub struct Provider {
    pub url: String,
    pub client: Client<OpenAIConfig>,
}

impl PartialEq for Provider {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }

    fn ne(&self, other: &Self) -> bool {
        self.url != other.url
    }
}

impl Into<Provider> for &SavedProvider {
    fn into(self) -> Provider {
        Provider {
            url: self.url.clone(),
            client: Client::with_config(
                OpenAIConfig::new()
                    .with_api_base(&self.url)
                    .with_api_key(&self.api_key),
            ),
        }
    }
}

impl Provider {
    pub fn is_usable(&self) -> bool {
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
        tokio_runtime.block_on(self.client.models().list()).is_ok()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SavedProvider {
    pub name: String,
    pub url: String,
    pub api_key: String,
    pub provider_type: ProviderType,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ProviderType {
    OpenAI,
    Gemini,
    Ollama,
}

impl SavedProviders {
    pub fn save(&self, path: &str) {
        let path = get_path_settings(path.to_string());
        let writer = File::create(path);

        if let Ok(writer) = writer {
            let _ = serde_json::to_writer_pretty(writer, &self);
        }
    }

    pub fn load(path: &str) -> Result<Self, String> {
        let path = get_path_settings(path.to_string());
        let reader = File::open(path);

        if let Ok(mut reader) = reader {
            let mut data = String::new();
            let _ = reader
                .read_to_string(&mut data)
                .map_err(|e| e.to_string())?;

            let de_data = serde_json::from_str::<Self>(&data);

            return match de_data {
                Ok(mut x) => {
                    if let None =
                        x.0.iter()
                            .find(|x| x.1.provider_type == ProviderType::Ollama)
                    {
                        if let Some(y) = get_local_ollama() {
                            x.0.insert(Id::new(), y);
                        }
                    }
                    Ok(x)
                }
                Err(e) => Err(e.to_string()),
            };
        }

        return Err("Failed to open file".to_string());
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PullModelStatus {
    #[serde(rename = "status")]
    pub message: String,
    pub digest: Option<String>,
    pub total: Option<u64>,
    pub completed: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
struct PullModelRequest {
    #[serde(rename = "name")]
    model_name: String,
    #[serde(rename = "insecure")]
    allow_insecure: bool,
    stream: bool,
}

pub type PullModelStatusStream =
    std::pin::Pin<Box<dyn Stream<Item = Result<PullModelStatus, String>> + Send>>;

impl Provider {
    pub async fn pull_ollama_model_stream(
        &self,
        model_name: String,
        allow_insecure: bool,
    ) -> Result<PullModelStatusStream, String> {
        let request = PullModelRequest {
            model_name,
            allow_insecure,
            stream: true,
        };

        let client = reqwest::Client::new();
        let url = format!("{}api/pull", self.url);
        let builder = client.post(url);
        let res = builder
            .json(&request)
            .send()
            .await
            .map_err(|x| x.to_string())?;

        if !res.status().is_success() {
            return Err(res.text().await.map_err(|x| x.to_string())?.into());
        }

        let stream = Box::new(res.bytes_stream().map(|res| match res {
            Ok(bytes) => {
                let res = serde_json::from_slice::<PullModelStatus>(&bytes);
                match res {
                    Ok(res) => Ok(res),
                    Err(e) => return Err(e.to_string().into()),
                }
            }
            Err(e) => Err(e.to_string().into()),
        }));

        Ok(std::pin::Pin::from(stream))
    }

    pub async fn get_models_async(&self) -> Vec<String> {
        return self
            .client
            .models()
            .list()
            .await
            .map(|x| x.data)
            .unwrap_or(Vec::new())
            .iter()
            .map(|x| x.id.clone())
            .collect::<Vec<String>>();
    }

    pub fn get_models(&self) -> Vec<String> {
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
        tokio_runtime.block_on(self.get_models_async())
    }
}
