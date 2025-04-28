use crate::utils::get_path_settings;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{fs::File, io::Read};

pub const SAVE_FILE: &str = "chat.json";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Save {
    pub theme: Option<usize>,
    pub use_panes: bool,
}

impl Default for Save{
    fn default() -> Self {
        Self {
            theme: None,
            use_panes: true,
        }
    }
}

impl Save {
    pub fn save(&self, path: &str) {
        let path = get_path_settings(path.to_string());
        let writer = File::create(path);

        if let Ok(writer) = writer {
            let _ = serde_json::to_writer_pretty(writer, &self);
        }
    }

    pub fn replace(&mut self, save: Save) {
        *self = save;
    }

    pub fn load(path: &str) -> Result<Self, String> {
        let path = get_path_settings(path.to_string());
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
