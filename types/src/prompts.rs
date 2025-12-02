use crate::surreal::RecordId;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Clone, Debug, Builder, Default)]
pub struct PromptData {
    pub command: String,
    pub title: String,
    pub content: String,
}

impl Display for Prompt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)
    }
}

impl Into<PromptData> for Prompt {
    fn into(self) -> PromptData {
        PromptData {
            command: self.command,
            title: self.title,
            content: self.content,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Prompt {
    pub command: String,
    pub title: String,
    pub content: String,
    pub id: RecordId,
}
