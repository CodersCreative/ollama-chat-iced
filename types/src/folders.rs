use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::surreal::RecordId;

#[derive(Serialize, Deserialize, Clone, Debug, Builder, Default)]
pub struct FolderData {
    #[builder(default = "None")]
    pub user_id: Option<RecordId>,
    #[builder(default = "Vec::new()")]
    #[serde(default = "Vec::new")]
    pub chats: Vec<String>,
    #[builder(default = "None")]
    pub parent: Option<String>,
    #[builder(default = "new_name()")]
    #[serde(default = "new_name")]
    pub name: String,
}

fn new_name() -> String {
    String::from("New Folder")
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Folder {
    pub user_id: RecordId,
    #[serde(default = "Vec::new")]
    pub chats: Vec<String>,
    pub parent: Option<String>,
    pub name: String,
    pub id: RecordId,
}
