use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::{env, fmt::Display, fs, path::PathBuf, str::FromStr};

use crate::surreal::RecordId;

#[derive(Serialize, Deserialize, Clone, Debug, Builder, Default)]
pub struct SettingsData {
    #[builder(default = "None")]
    pub previews_provider: Option<SettingsProvider>,
    #[builder(default = "None")]
    pub models_path: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder, PartialEq, Eq, PartialOrd, Ord)]
pub struct SettingsProvider {
    pub provider: String,
    pub model: String,
}

impl Display for SettingsProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.model)
    }
}

impl Into<SettingsData> for Settings {
    fn into(self) -> SettingsData {
        SettingsData {
            previews_provider: self.previews_provider,
            models_path: Some(self.models_path),
        }
    }
}

fn get_path_local(path: String) -> String {
    let mut new_path = env::var("XDG_CONFIG_HOME")
        .or_else(|_| env::var("HOME"))
        .unwrap();
    new_path.push_str(&format!("/.local/share/ochat"));

    if !fs::exists(&new_path).unwrap_or(true) {
        fs::create_dir(&new_path).unwrap();
    }

    new_path.push_str(&format!("/{}", path));
    return new_path;
}

fn get_models_path() -> PathBuf {
    PathBuf::from_str(&get_path_local("models/".to_string())).unwrap()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub previews_provider: Option<SettingsProvider>,
    #[serde(default = "get_models_path")]
    pub models_path: PathBuf,
    pub id: RecordId,
}
