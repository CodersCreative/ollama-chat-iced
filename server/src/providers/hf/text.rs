use super::API_URL;
use crate::errors::ServerError;
use axum::{Json, extract::Path};
use ochat_types::providers::hf::HFModel;
use serde::Deserialize;
use serde_json::Value;

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
