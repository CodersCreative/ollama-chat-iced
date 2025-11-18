use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::surreal::RecordId;

#[derive(Serialize, Deserialize, Clone, Debug, Builder, Default)]
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
    pub id: RecordId,
}
