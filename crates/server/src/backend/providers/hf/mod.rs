use crate::backend::{errors::ServerError, settings::get_settings};
use axum::Json;
use ochat_types::providers::hf::{
    DownloadedHFModels, HFModelDetails, HFModelVariant, HFModelVariants, ModelType,
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use tokio::fs;

pub mod pull;
pub mod stt;
pub mod text;

const HF_URL: &str = "https://huggingface.co";
const API_URL: &str = "https://huggingface.co/api";

pub async fn get_downloaded_hf_models() -> Result<Json<DownloadedHFModels>, ServerError> {
    let directory = get_settings().await?.0.models_path;
    let mut files = Vec::new();
    let mut list = fs::read_dir(directory).await?;

    while let Some(sub_type) = list.next_entry().await? {
        if !sub_type.file_type().await?.is_dir() {
            continue;
        }
        let mut directory = fs::read_dir(sub_type.path()).await?;
        let sub_type = sub_type.file_name().display().to_string();

        while let Some(author) = directory.next_entry().await? {
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
                        || ["bin", "gguf", "safetensors"].contains(
                            &file
                                .path()
                                .extension()
                                .unwrap_or_default()
                                .display()
                                .to_string()
                                .trim(),
                        )
                    {
                        continue;
                    }

                    files.push(HFModelVariant {
                        model_type: if &sub_type == "speech" {
                            ModelType::Speech
                        } else {
                            ModelType::Text
                        },
                        model: format!(
                            "{}/{}",
                            author.file_name().display(),
                            model.file_name().display(),
                        ),
                        name: file.file_name().display().to_string(),
                        size: Some(file.metadata().await?.len()),
                        is_sharded: false,
                    });
                }
            }
        }
    }

    Ok(Json(DownloadedHFModels { variants: files }))
}
pub async fn get_variants_base(
    id: String,
    client: &reqwest::Client,
    desired_file_type: &str,
    model_type: ModelType,
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

    let is_sharded = desired_file_type == "safetensors"
        && entries
            .iter()
            .any(|e| e.path.ends_with(".safetensors.index.json"));

    for entry in entries {
        if entry.r#type != "file" || !entry.path.ends_with(desired_file_type) {
            continue;
        }

        let path_lower = entry.path.to_lowercase();

        let precision: u64 = if path_lower.contains("f32") || path_lower.contains("fp32") {
            32
        } else if path_lower.contains("f16")
            || path_lower.contains("fp16")
            || path_lower.contains("bf16")
        {
            16
        } else if path_lower.contains("q8") {
            8
        } else if path_lower.contains("q4") {
            4
        } else {
            let file_stem = entry
                .path
                .trim_end_matches(&format!(".{}", desired_file_type));
            let variant = file_stem.rsplit(['-', '.']).next().unwrap_or(file_stem);
            variant
                .split('_')
                .next()
                .unwrap_or(variant)
                .trim_start_matches(|c: char| !c.is_numeric())
                .parse()
                .unwrap_or_default()
        };

        let variant_list = files.entry(precision).or_default();

        variant_list.push(HFModelVariant {
            model: id.clone(),
            name: entry.path,
            model_type: model_type.clone(),
            size: Some(entry.size),
            is_sharded,
        });
    }

    Ok(HFModelVariants(files))
}

pub async fn fetch_model_details_base(
    user: String,
    id: String,
    desired_file_type: &str,
    model_type: ModelType,
) -> Result<Json<HFModelDetails>, ServerError> {
    let id = format!("{}/{}", user.trim(), id.trim());
    let client = reqwest::Client::new();
    let request = client.get(format!("{}/models/{}", API_URL, id.trim()));
    let response: Value = request.send().await?.error_for_status()?.json().await?;
    let params: u64 = if let Some(x) = response.get(desired_file_type) {
        x.get("total").unwrap().as_u64().unwrap_or_default()
    } else {
        0
    };

    let mut model: HFModelDetails = serde_json::from_value(response).unwrap();
    model.parameters = params;
    model.description = reqwest::get(format!("{}/{}/raw/main/README.md", HF_URL, id.trim()))
        .await?
        .text()
        .await?;
    model.id = id.clone();
    model.variants = get_variants_base(id, &client, desired_file_type, model_type).await?;

    Ok(Json(model))
}
