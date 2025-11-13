pub mod models;
pub mod ollama;

use async_openai::{Client, config::OpenAIConfig};
use axum::{Json, extract::Path};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

use crate::{CONN, errors::ServerError};
pub const PROVIDER_TABLE: &str = "providers";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum ProviderType {
    OpenAI,
    Gemini,
    #[default]
    Ollama,
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct ProviderData {
    pub name: String,
    pub url: String,
    pub api_key: String,
    #[builder(default = "ProviderType::Ollama")]
    pub provider_type: ProviderType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Provider {
    pub name: String,
    pub url: String,
    pub api_key: String,
    pub provider_type: ProviderType,
    pub id: RecordId,
}

impl Into<Client<OpenAIConfig>> for &Provider {
    fn into(self) -> Client<OpenAIConfig> {
        Client::with_config(
            OpenAIConfig::new()
                .with_api_base(&self.url)
                .with_api_key(&self.api_key),
        )
    }
}
async fn get_local_ollama_data() -> Option<ProviderData> {
    let url = "http://localhost:11434/v1".to_string();
    if reqwest::Client::new().head(&url).send().await.is_ok() {
        Some(ProviderData {
            name: String::from("Local Ollama"),
            url,
            api_key: String::from("ollama"),
            provider_type: ProviderType::Ollama,
        })
    } else {
        None
    }
}

pub async fn define_providers() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS name ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS url ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS api_key ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS provider_type ON TABLE {0} TYPE string;
",
            PROVIDER_TABLE
        ))
        .await?;

    if match list_all_providers().await {
        Ok(x) => x.is_empty(),
        _ => true,
    } {
        if let Some(ollama) = get_local_ollama_data().await {
            let _ = add_provider(Json(ollama)).await?;
        }
    }

    Ok(())
}

pub async fn add_provider(
    Json(provider): Json<ProviderData>,
) -> Result<Json<Option<Provider>>, ServerError> {
    let provider: Option<Provider> = CONN.create(PROVIDER_TABLE).content(provider).await?;

    Ok(Json(provider))
}

pub async fn read_provider(id: Path<String>) -> Result<Json<Option<Provider>>, ServerError> {
    let provider = CONN.select((PROVIDER_TABLE, &*id)).await?;
    Ok(Json(provider))
}

pub async fn update_provider(
    id: Path<String>,
    Json(provider): Json<ProviderData>,
) -> Result<Json<Option<Provider>>, ServerError> {
    let provider: Option<Provider> = CONN
        .update((PROVIDER_TABLE, &*id))
        .content(provider)
        .await?;

    Ok(Json(provider))
}

pub async fn delete_provider(id: Path<String>) -> Result<Json<Option<Provider>>, ServerError> {
    let provider: Option<Provider> = CONN.delete((PROVIDER_TABLE, &*id)).await?;

    Ok(Json(provider))
}

pub async fn list_all_providers() -> Result<Json<Vec<Provider>>, ServerError> {
    let providers = CONN.select(PROVIDER_TABLE).await?;
    Ok(Json(providers))
}
