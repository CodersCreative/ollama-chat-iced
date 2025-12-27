pub mod generic_rig;
pub mod hf;
pub mod models;
pub mod ollama;
pub mod route;

use axum::{Json, extract::Path, http::HeaderMap};
use ochat_types::providers::*;

use crate::backend::{CONN, errors::ServerError};
pub const PROVIDER_TABLE: &str = "providers";

pub(crate) fn provider_into_config(provider: &Provider) -> generic_rig::Client {
    generic_rig::Client::builder()
        .base_url(provider.url.trim())
        .api_key(provider.api_key.trim())
        .build()
        .unwrap()
}

pub(crate) fn provider_into_reqwest(provider: &Provider) -> reqwest::ClientBuilder {
    let mut header = HeaderMap::new();
    if !provider.api_key.is_empty() {
        header.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&provider.api_key).unwrap(),
        );
    }

    reqwest::Client::builder().default_headers(header)
}

async fn get_local_ollama_data() -> Option<ProviderData> {
    let url = "http://localhost:11434/v1".to_string();
    if reqwest::Client::new().get(&url).send().await.is_ok() {
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
DEFINE TABLE IF NOT EXISTS {0} SCHEMAFULL PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS name ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS url ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS api_key ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS provider_type ON TABLE {0} TYPE string;
",
            PROVIDER_TABLE
        ))
        .await?;

    Ok(())
}

pub async fn add_default_providers() -> Result<(), ServerError> {
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
    let provider = CONN.select((PROVIDER_TABLE, id.trim())).await?;
    Ok(Json(provider))
}

pub async fn update_provider(
    id: Path<String>,
    Json(provider): Json<ProviderData>,
) -> Result<Json<Option<Provider>>, ServerError> {
    let provider: Option<Provider> = CONN
        .update((PROVIDER_TABLE, id.trim()))
        .content(provider)
        .await?;

    Ok(Json(provider))
}

pub async fn delete_provider(id: Path<String>) -> Result<Json<Option<Provider>>, ServerError> {
    let provider: Option<Provider> = CONN.delete((PROVIDER_TABLE, id.trim())).await?;

    Ok(Json(provider))
}

pub async fn list_all_providers() -> Result<Json<Vec<Provider>>, ServerError> {
    let providers = CONN.select(PROVIDER_TABLE).await?;
    Ok(Json(providers))
}
