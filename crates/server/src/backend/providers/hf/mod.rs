use crate::backend::{errors::ServerError, settings::get_settings};
use axum::Json;
use ochat_types::providers::hf::{
    DownloadedHFModels, HFModel, HFModelDetails, HFModelVariant, HFModelVariants, ModelType,
};
use serde::Deserialize;
use serde_json::Value;
use spider::hashbrown::HashSet;
use std::collections::HashMap;
use tokio::fs;

pub mod conversion;
pub mod pull;
pub mod stt;
pub mod text;
pub mod tts;

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

                while let Some(name) = directory.next_entry().await? {
                    if !name.file_type().await?.is_dir() {
                        continue;
                    }
                    let mut directory = fs::read_dir(name.path()).await?;

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
                            model_type: if &sub_type == "stt" {
                                ModelType::Stt
                            } else if &sub_type == "tts" {
                                ModelType::Tts
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
    }

    Ok(Json(DownloadedHFModels { variants: files }))
}

pub fn process_searched_models(mut models: Vec<Value>) -> Result<Json<Vec<HFModel>>, ServerError> {
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

pub async fn model_is_gated(id: &str, client: &reqwest::Client) -> Option<bool> {
    let Ok(request) = client
        .get(format!("{}/models/{}", API_URL, id.trim()))
        .send()
        .await
    else {
        return None;
    };

    let Ok(value) = request.json::<Value>().await else {
        return None;
    };

    let Some(value) = value.get("gated") else {
        return None;
    };

    value.as_bool()
}

pub async fn get_variants_base(
    id: String,
    client: &reqwest::Client,
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
    let mut processed_base_models: HashSet<String> = HashSet::new();

    let is_sharded = entries
        .iter()
        .any(|e| e.path.ends_with(".safetensors.index.json"));

    for entry in entries {
        if entry.r#type != "file"
            || (!entry.path.ends_with(".bin")
                && !entry.path.ends_with(".ggml")
                && !entry.path.ends_with(".gguf")
                && !entry.path.ends_with(".safetensors"))
            || entry.path.contains("pytorch")
        {
            continue;
        }

        let path_lower = entry.path.to_lowercase();

        if entry.path.contains("-of-") {
            let base_stem = entry
                .path
                .split("-00")
                .next()
                .unwrap_or(&entry.path)
                .to_string();
            if processed_base_models.contains(&base_stem) {
                continue;
            }
            processed_base_models.insert(base_stem);
        }

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
                .trim_end_matches(".bin")
                .trim_end_matches(".ggml")
                .trim_end_matches(".gguf")
                .trim_end_matches(".safetensors");
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
    model_type: ModelType,
) -> Result<Json<HFModelDetails>, ServerError> {
    let id = format!("{}/{}", user.trim(), id.trim());
    let client = reqwest::Client::new();
    let request = client.get(format!("{}/models/{}", API_URL, id.trim()));
    let response: Value = request.send().await?.error_for_status()?.json().await?;
    let params: u64 = if let Some(x) = response.get("gguf") {
        x.get("total").unwrap().as_u64().unwrap_or_default()
    } else if let Some(x) = response.get("ggml") {
        x.get("total").unwrap().as_u64().unwrap_or_default()
    } else if let Some(x) = response.get("bin") {
        x.get("total").unwrap().as_u64().unwrap_or_default()
    } else if let Some(x) = response.get("safetensors") {
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
    model.variants = get_variants_base(id, &client, model_type).await?;

    Ok(Json(model))
}
