use std::{
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

use async_openai::{Client, config::OpenAIConfig};
use axum::{Json, extract::Path};
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

use crate::{CONN, errors::ServerError};

const PROVIDER_TABLE: &str = "providers";
pub static PROVIDERS: LazyLock<RwLock<HashMap<String, RuntimeProvider>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

#[derive(Debug, Clone)]
pub struct RuntimeProvider {
    pub url: String,
    pub client: Client<OpenAIConfig>,
}

unsafe impl Send for RuntimeProvider {}
unsafe impl Sync for RuntimeProvider {}

impl PartialEq for RuntimeProvider {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }

    fn ne(&self, other: &Self) -> bool {
        self.url != other.url
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ProviderType {
    OpenAI,
    Gemini,
    Ollama,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProviderData {
    pub name: String,
    pub url: String,
    pub api_key: String,
    pub provider_type: ProviderType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Provider {
    pub name: String,
    pub url: String,
    pub api_key: String,
    pub provider_type: ProviderType,
    id: RecordId,
}

impl Into<RuntimeProvider> for &Provider {
    fn into(self) -> RuntimeProvider {
        RuntimeProvider {
            url: self.url.clone(),
            client: Client::with_config(
                OpenAIConfig::new()
                    .with_api_base(&self.url)
                    .with_api_key(&self.api_key),
            ),
        }
    }
}

pub async fn define_chat() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS name ON TABLE {0} TYPE string UNIQUE;
DEFINE FIELD IF NOT EXISTS url ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS api_key ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS provider_type ON TABLE {0} TYPE string;
",
            PROVIDER_TABLE
        ))
        .await?;
    Ok(())
}

pub async fn add_provider(
    Json(provider): Json<ProviderData>,
) -> Result<Json<Option<Provider>>, ServerError> {
    let provider: Option<Provider> = CONN.create(PROVIDER_TABLE).content(provider).await?;

    if let Some(provider) = &provider {
        PROVIDERS
            .write()
            .unwrap()
            .insert(provider.id.key().to_string(), provider.into());
    }

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

    if let Some(provider) = &provider {
        PROVIDERS
            .write()
            .unwrap()
            .insert(provider.id.key().to_string(), provider.into());
    }

    Ok(Json(provider))
}

pub async fn delete_provider(id: Path<String>) -> Result<Json<Option<Provider>>, ServerError> {
    let provider: Option<Provider> = CONN.delete((PROVIDER_TABLE, &*id)).await?;

    if let Some(provider) = &provider {
        PROVIDERS
            .write()
            .unwrap()
            .remove(&provider.id.key().to_string());
    }

    Ok(Json(provider))
}

pub async fn list_all_providers() -> Result<Json<Vec<Provider>>, ServerError> {
    let providers = CONN.select(PROVIDER_TABLE).await?;
    Ok(Json(providers))
}
