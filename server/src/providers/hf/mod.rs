use axum::{Json, extract::Path};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use tokio::fs;

use ochat_types::providers::hf::{
    DownloadedHFModels, HFModelDetails, HFModelVariant, HFModelVariants,
};

use crate::{errors::ServerError, settings::get_settings};

pub mod pull;
pub mod text;

const HF_URL: &str = "https://huggingface.co";
const API_URL: &str = "https://huggingface.co/api";

pub async fn get_downloaded_hf_models() -> Result<Json<DownloadedHFModels>, ServerError> {
    let directory = get_settings().await?.0.models_path;
    let mut files = Vec::new();
    let mut list = fs::read_dir(directory).await?;

    while let Some(author) = list.next_entry().await? {
        if !author.file_type().await?.is_dir() {
            continue;
        }

        let mut directory = fs::read_dir(author.path()).await?;

        while let Some(model) = directory.next_entry().await? {
            if !model.file_type().await?.is_dir() {
                continue;
            }

            let mut directory = fs::read_dir(model.path()).await?;

            while let Some(file) = directory.next_entry().await? {
                if !file.file_type().await?.is_file()
                    || file.path().extension().unwrap_or_default() != "gguf"
                {
                    continue;
                }

                files.push(HFModelVariant {
                    model: format!(
                        "{}/{}",
                        author.file_name().display(),
                        model.file_name().display(),
                    ),
                    name: file.file_name().display().to_string(),
                    size: Some(file.metadata().await?.len()),
                });
            }
        }
    }

    Ok(Json(DownloadedHFModels { variants: files }))
}

pub async fn fetch_model_details(
    Path((user, id)): Path<(String, String)>,
) -> Result<Json<HFModelDetails>, ServerError> {
    let id = format!("{}/{}", user.trim(), id.trim());
    let client = reqwest::Client::new();
    let request = client.get(format!("{}/models/{}", API_URL, id.trim()));
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
    model.description = reqwest::get(format!("{}/{}/raw/main/README.md", HF_URL, id.trim()))
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
    let request = client.get(format!("{}/models/{}/tree/main", API_URL, id.trim()));

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
