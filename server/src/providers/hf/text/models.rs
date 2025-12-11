use super::API_URL;
use crate::{errors::ServerError, providers::hf::text::HF_URL};
use axum::{Json, extract::Path};
use ochat_types::providers::hf::{HFModel, HFModelDetails, HFModelVariant, HFModelVariants};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

pub async fn search_models(search: Path<String>) -> Result<Json<Vec<HFModel>>, ServerError> {
    let client = reqwest::Client::new();

    let request = client.get(format!("{API_URL}/models")).query(&[
        ("search", search.as_str()),
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

pub async fn fetch_model_details(
    Path((user, id)): Path<(String, String)>,
) -> Result<Json<HFModelDetails>, ServerError> {
    let id = format!("{user}/{id}");
    let client = reqwest::Client::new();
    let request = client.get(format!("{}/models/{}", API_URL, &id));
    let response: Value = request.send().await?.error_for_status()?.json().await?;
    let params: u64 = response
        .get("gguf")
        .unwrap()
        .get("total")
        .unwrap()
        .as_u64()
        .unwrap_or_default();
    let mut model: HFModelDetails = serde_json::from_value(response).unwrap();
    model.parameters = params;
    model.description = reqwest::get(format!("{}/{}/raw/main/README.md", HF_URL, &id))
        .await?
        .text()
        .await?;
    model.id = id.clone();
    model.variants = get_variants(id, &client).await?;

    Ok(Json(model))
}

async fn get_variants(
    id: String,
    client: &reqwest::Client,
) -> Result<HFModelVariants, ServerError> {
    let request = client.get(format!("{}/models/{}/tree/main", API_URL, id));

    #[derive(Debug, Deserialize)]
    struct Entry {
        r#type: String,
        path: String,
        size: u64,
    }

    let entries: Vec<Entry> = request.send().await?.error_for_status()?.json().await?;
    let mut files: HashMap<u64, Vec<HFModelVariant>> = HashMap::new();

    for entry in entries {
        if entry.r#type != "file" || !entry.path.ends_with(".gguf") {
            continue;
        }

        let file_stem = entry.path.trim_end_matches(".gguf");
        let variant = file_stem.rsplit(['-', '.']).next().unwrap_or(file_stem);
        let precision = variant
            .split('_')
            .next()
            .unwrap_or(variant)
            .trim_start_matches("IQ")
            .trim_start_matches("Q")
            .trim_start_matches("BF")
            .trim_start_matches("F")
            .parse();

        let Ok(precision) = precision else {
            continue;
        };

        let files = files.entry(precision).or_default();

        files.push(HFModelVariant {
            model: id.clone(),
            name: entry.path,
            size: Some(entry.size),
        })
    }

    Ok(HFModelVariants(files))
}
