use axum::{Json, extract::Path};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

use crate::{
    CONN,
    errors::ServerError,
    providers::{ProviderType, list_all_providers, models::list_all_provider_models},
};

const SETTINGS_TABLE: &str = "settings";

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct SettingsData {
    #[builder(default = "None")]
    pub previews_provider: Option<SettingsProvider>,
    #[builder(default = "None")]
    pub default_provider: Option<SettingsProvider>,
    #[builder(default = "None")]
    pub tools_provider: Option<SettingsProvider>,
    #[builder(default = "None")]
    pub use_panes: Option<bool>,
    #[builder(default = "None")]
    pub theme: Option<usize>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct SettingsProvider {
    pub provider: String,
    pub model: String,
}

impl Into<SettingsData> for Settings {
    fn into(self) -> SettingsData {
        SettingsData {
            previews_provider: self.previews_provider,
            default_provider: self.default_provider,
            tools_provider: self.tools_provider,
            use_panes: Some(self.use_panes),
            theme: Some(self.theme),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub previews_provider: Option<SettingsProvider>,
    pub default_provider: Option<SettingsProvider>,
    pub tools_provider: Option<SettingsProvider>,
    pub use_panes: bool,
    pub theme: usize,
    id: RecordId,
}

pub async fn define_settings() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS previews_provider ON TABLE {0} TYPE option<object>;
DEFINE FIELD IF NOT EXISTS default_provider ON TABLE {0} TYPE option<object>;
DEFINE FIELD IF NOT EXISTS tools_provider ON TABLE {0} TYPE option<object>;
DEFINE FIELD IF NOT EXISTS use_panes ON TABLE {0} TYPE bool;
DEFINE FIELD IF NOT EXISTS theme ON TABLE {0} TYPE int;
",
            SETTINGS_TABLE,
        ))
        .await?;
    Ok(())
}

pub async fn reset_settings() -> Result<Json<Option<Settings>>, ServerError> {
    let default_provider: Option<SettingsProvider> = {
        let mut providers = list_all_providers().await?.0;
        let provider = if let Some(x) = providers
            .iter()
            .find(|x| x.provider_type == ProviderType::Ollama)
        {
            Some(x.clone())
        } else {
            Some(providers.remove(0))
        };

        if let Some(provider) = provider {
            let mut models = list_all_provider_models(Path(provider.id.key().to_string()))
                .await?
                .0;

            if !models.is_empty() {
                Some(
                    SettingsProviderBuilder::default()
                        .provider(provider.id.key().to_string())
                        .model(models.remove(0).id)
                        .build()
                        .unwrap(),
                )
            } else {
                None
            }
        } else {
            None
        }
    };

    let settings = SettingsData {
        previews_provider: default_provider.clone(),
        tools_provider: default_provider.clone(),
        default_provider,
        use_panes: Some(true),
        theme: Some(11),
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
        reset_settings().await?.0.unwrap()
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

    if let Some(x) = settings.default_provider {
        current_settings.default_provider = Some(x);
    }

    if let Some(x) = settings.tools_provider {
        current_settings.tools_provider = Some(x);
    }

    if let Some(x) = settings.use_panes {
        current_settings.use_panes = x;
    }

    if let Some(x) = settings.theme {
        current_settings.theme = x;
    }

    let chat = CONN
        .update((SETTINGS_TABLE, current_settings.id.key().to_string()))
        .content(Into::<SettingsData>::into(current_settings))
        .await?;
    Ok(Json(chat))
}
