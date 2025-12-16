use std::{fs::File, io::Read};

use ochat_types::settings::SettingsProvider;
use serde::{Deserialize, Serialize};

use crate::utils::get_path_settings;

pub const SETTINGS_PATH: &str = "settings.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClientSettings {
    pub instance_url: String,
    pub default_provider: Option<SettingsProvider>,
    pub use_panes: bool,
    pub theme: usize,
}
impl Default for ClientSettings {
    fn default() -> Self {
        ClientSettings {
            instance_url: String::from("http://localhost:1212"),
            default_provider: None,
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
        let reader = File::open(path);

        if let Ok(mut reader) = reader {
            let mut data = String::new();
            let _ = reader
                .read_to_string(&mut data)
                .map_err(|e| e.to_string())?;

            let de_data = serde_json::from_str(&data);

            return match de_data {
                Ok(x) => Ok(x),
                Err(e) => Err(e.to_string()),
            };
        }

        return Err("Failed to open file".to_string());
    }
}
