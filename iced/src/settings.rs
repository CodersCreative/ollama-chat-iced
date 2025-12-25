use ochat_common::{get_path_settings, load_from_file};
use ochat_types::settings::SettingsProvider;
use serde::{Deserialize, Serialize};
use std::fs::File;

pub const SETTINGS_PATH: &str = "settings.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClientSettings {
    pub instance_url: String,
    pub default_provider: Option<SettingsProvider>,
    pub default_tools: Vec<String>,
    pub use_panes: bool,
    pub theme: usize,
}
impl Default for ClientSettings {
    fn default() -> Self {
        ClientSettings {
            instance_url: String::from("http://localhost:1212/api"),
            default_provider: None,
            default_tools: Vec::new(),
            use_panes: true,
            theme: 11,
        }
    }
}

impl ClientSettings {
    pub fn save(&self) {
        let path = get_path_settings(SETTINGS_PATH.to_string());
        let writer = File::create(path);

        if let Ok(writer) = writer {
            let _ = serde_json::to_writer_pretty(writer, &self);
        }
    }

    pub fn load() -> Result<Self, String> {
        let path = get_path_settings(SETTINGS_PATH.to_string());
        load_from_file(&path)
    }
}
