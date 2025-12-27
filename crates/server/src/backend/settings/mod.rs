use crate::backend::{
    CONN,
    errors::ServerError,
    providers::{list_all_providers, models::list_all_provider_models},
    utils::get_path_local,
};
use axum::{Json, extract::Path};
use ochat_types::{providers::ProviderType, settings::*};
use std::{path::PathBuf, str::FromStr};

pub mod route;
const SETTINGS_TABLE: &str = "settings";

pub async fn define_settings() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS previews_provider ON TABLE {0} TYPE option<object>;
DEFINE FIELD IF NOT EXISTS models_path ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS use_llama_cpp ON TABLE {0} TYPE bool;
",
            SETTINGS_TABLE,
        ))
        .await?;
    Ok(())
}

pub async fn reset_settings() -> Result<Json<Option<Settings>>, ServerError> {
    let (embeddings_provider, default_provider) = {
        let mut providers = list_all_providers().await.unwrap_or_default().0;
        let provider = if let Some(x) = providers
            .iter()
            .find(|x| x.provider_type == ProviderType::Ollama)
        {
            Some(x.clone())
        } else if providers.len() > 0 {
            Some(providers.remove(0))
        } else {
            None
        };

        if let Some(provider) = provider {
            let mut models = list_all_provider_models(Path(provider.id.key().to_string()))
                .await?
                .0;

            if !models.is_empty() {
                (
                    models.iter().find(|x| x.id.contains("embed")).map(|x| {
                        SettingsProviderBuilder::default()
                            .provider(provider.id.key().to_string())
                            .model(x.id.clone())
                            .build()
                            .unwrap()
                    }),
                    Some(
                        SettingsProviderBuilder::default()
                            .provider(provider.id.key().to_string())
                            .model(models.remove(0).id)
                            .build()
                            .unwrap(),
                    ),
                )
            } else {
                (None, None)
            }
        } else {
            (None, None)
        }
    };

    let settings = SettingsData {
        previews_provider: default_provider,
        embeddings_provider,
        use_llama_cpp: Some(true),
        models_path: Some(PathBuf::from_str(&get_path_local("models/".to_string())).unwrap()),
    };

    let settings_list: Vec<Settings> = CONN.select(SETTINGS_TABLE).await?;

    Ok(Json(if settings_list.is_empty() {
        CONN.create(SETTINGS_TABLE).content(settings).await?
    } else {
        CONN.update((SETTINGS_TABLE, settings_list[0].id.key().to_string()))
            .content(settings)
            .await?
    }))
}

pub async fn get_settings() -> Result<Json<Settings>, ServerError> {
    let mut settings: Vec<Settings> = CONN.select(SETTINGS_TABLE).await?;

    Ok(Json(if settings.is_empty() {
        reset_settings().await?.0.unwrap_or(Settings {
            previews_provider: None,
            embeddings_provider: None,
            use_llama_cpp: true,
            models_path: PathBuf::from_str(&get_path_local("models/".to_string())).unwrap(),
            id: (SETTINGS_TABLE, "unknown").into(),
        })
    } else {
        settings.remove(0)
    }))
}

pub async fn update_settings(
    Json(settings): Json<SettingsData>,
) -> Result<Json<Option<Settings>>, ServerError> {
    let mut current_settings = get_settings().await?.0;

    if let Some(x) = settings.previews_provider {
        current_settings.previews_provider = Some(x);
    }

    if let Some(x) = settings.models_path {
        current_settings.models_path = x;
    }

    if let Some(x) = settings.use_llama_cpp {
        current_settings.use_llama_cpp = x;
    }

    let chat: Vec<Settings> = CONN
        .update(SETTINGS_TABLE)
        .content(Into::<SettingsData>::into(current_settings))
        .await?;

    Ok(Json(chat.first().map(|x| x.clone())))
}
