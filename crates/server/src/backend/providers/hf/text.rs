use super::API_URL;
use crate::backend::{errors::ServerError, settings::get_settings};
use axum::{Json, extract::Path};
use ochat_types::{providers::hf::HFModel, settings::SettingsProvider};
use serde::Deserialize;
use serde_json::Value;
use tokio::fs;

pub async fn search_models(search: Path<String>) -> Result<Json<Vec<HFModel>>, ServerError> {
    let client = reqwest::Client::new();

    let request = client.get(format!("{API_URL}/models")).query(&[
        ("search", search.trim()),
        ("filter", "text-generation"),
        ("filter", "gguf"),
        ("limit", "100"),
        ("full", "true"),
    ]);

    let response = request.send().await?;
    let mut models: Vec<Value> = response.json().await?;

    #[derive(Deserialize, PartialEq, Eq)]
    #[serde(untagged)]
    enum Gated {
        Bool(bool),
        Other(String),
    }

    impl Default for Gated {
        fn default() -> Self {
            Gated::Bool(true)
        }
    }

    models.retain(|model| {
        Gated::Bool(false)
            == serde_json::from_value(model.get("gated").unwrap().clone()).unwrap_or_default()
    });

    Ok(Json(
        models
            .into_iter()
            .filter_map(|x| match serde_json::from_value(x) {
                Ok(x) => Some(x),
                _ => None,
            })
            .collect(),
    ))
}

pub async fn list_all_models() -> Result<Json<Vec<HFModel>>, ServerError> {
    search_models(Path(String::new())).await
}

pub async fn list_all_downloaded_models() -> Result<Json<Vec<SettingsProvider>>, ServerError> {
    let dir = get_settings().await.unwrap().models_path.clone();

    if !fs::try_exists(&dir).await.unwrap_or(true) {
        let _ = fs::create_dir(&dir).await.unwrap();
    }

    let dir = dir.join("text/");

    if !fs::try_exists(&dir).await.unwrap_or(true) {
        let _ = fs::create_dir(&dir).await.unwrap();
    }

    let mut models = Vec::new();

    let mut dir = fs::read_dir(dir).await?;
    while let Some(first) = dir.next_entry().await? {
        let user = first.file_name().into_string().unwrap().trim().to_string();
        let mut dir = fs::read_dir(first.path()).await?;

        while let Some(second) = dir.next_entry().await? {
            let model = second.file_name().into_string().unwrap().trim().to_string();
            let mut dir = fs::read_dir(second.path()).await?;

            while let Some(third) = dir.next_entry().await? {
                let name = third.file_name().into_string().unwrap().trim().to_string();
                if name.to_lowercase().ends_with("gguf") {
                    models.push(SettingsProvider {
                        provider: format!("HF:{}/{}", user, model),
                        model: name,
                    });
                }
            }
        }
    }

    Ok(Json(models))
}
