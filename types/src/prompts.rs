use crate::surreal::RecordId;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Clone, Debug, Builder, Default)]
pub struct PromptData {
    #[builder(default = "None")]
    pub user_id: Option<RecordId>,
    pub command: String,
    pub title: String,
    pub content: String,
    #[builder(default = "None")]
    pub downloads: Option<i16>,
    #[builder(default = "None")]
    pub upvotes: Option<i16>,
    #[builder(default = "None")]
    pub downvotes: Option<i16>,
    #[builder(default = "None")]
    pub user: Option<OpenWebUIUser>,
}

impl Display for Prompt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)
    }
}

impl Into<PromptData> for Prompt {
    fn into(self) -> PromptData {
        PromptData {
            user_id: Some(self.user_id),
            command: self.command,
            title: self.title,
            content: self.content,
            downloads: self.downloads,
            upvotes: self.upvotes,
            downvotes: self.downvotes,
            user: self.user,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Prompt {
    pub user_id: RecordId,
    pub command: String,
    pub title: String,
    pub content: String,
    pub downloads: Option<i16>,
    pub upvotes: Option<i16>,
    pub downvotes: Option<i16>,
    pub user: Option<OpenWebUIUser>,
    pub id: RecordId,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OpenWebUIUser {
    pub username: String,
    pub verified: bool,
}
