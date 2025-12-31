use super::API_URL;
use crate::backend::{
    errors::ServerError,
    providers::hf::{fetch_model_details_base, process_searched_models, pull::EXTRA_EXTS},
    settings::get_settings,
};
use axum::{Json, extract::Path};
use ochat_types::{
    providers::hf::{HFModel, HFModelDetails, ModelType},
    settings::SettingsProvider,
};
use tokio::fs;

pub async fn search_models(search: Path<String>) -> Result<Json<Vec<HFModel>>, ServerError> {
    let client = reqwest::Client::new();

    let request = client.get(format!("{API_URL}/models")).query(&[
        (
            "search",
            if search.contains("parler") {
                search.to_string()
            } else {
                format!("parler {}", search.trim())
            }
            .trim(),
        ),
        ("filter", "text-to-speech"),
        ("limit", "75"),
        ("full", "true"),
    ]);

    let response = request.send().await?;
    process_searched_models(response.json().await?)
}

pub async fn list_all_models() -> Result<Json<Vec<HFModel>>, ServerError> {
    search_models(Path(String::new())).await
}

pub async fn list_all_downloaded_models() -> Result<Json<Vec<SettingsProvider>>, ServerError> {
    let dir = get_settings().await.unwrap().models_path.clone();

    if !fs::try_exists(&dir).await.unwrap_or(true) {
        let _ = fs::create_dir(&dir).await.unwrap();
    }

    let dir = dir.join("tts/");

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
                let mut dir = fs::read_dir(third.path()).await?;

                while let Some(forth) = dir.next_entry().await? {
                    let name = forth.file_name().into_string().unwrap().trim().to_string();
                    if !EXTRA_EXTS.contains(&name.to_lowercase().rsplit_once(".").unwrap().1.trim())
                    {
                        models.push(SettingsProvider {
                            provider: format!("HF-TTS:{}/{}", user, model),
                            model: name,
                        });
                    }
                }
            }
        }
    }

    Ok(Json(models))
}

pub async fn fetch_model_details(
    Path((user, id)): Path<(String, String)>,
) -> Result<Json<HFModelDetails>, ServerError> {
    fetch_model_details_base(user, id, ModelType::Tts).await
}
