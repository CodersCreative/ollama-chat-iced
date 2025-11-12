pub mod messages;
pub mod previews;
pub mod relationships;

use axum::{Json, extract::Path};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use surrealdb::{Datetime, RecordId};

use crate::{CONN, errors::ServerError};

const CHAT_TABLE: &str = "chats";

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct ChatData {
    #[serde(default = "Vec::new")]
    #[builder(default = "Vec::new()")]
    pub default_chats: Vec<usize>,
    #[serde(default = "Vec::new")]
    #[builder(default = "Vec::new()")]
    pub default_tools: Vec<String>,
    #[builder(default = "None")]
    pub time: Option<Datetime>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Chat {
    #[serde(default = "Vec::new")]
    pub default_chats: Vec<usize>,
    #[serde(default = "Vec::new")]
    pub default_tools: Vec<String>,
    time: Datetime,
    id: RecordId,
}

pub async fn define_chats() -> Result<(), ServerError> {
    // Use a linking table for default_tools and chats
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS default_chats ON TABLE {0} TYPE array<int>;
DEFINE FIELD IF NOT EXISTS time ON TABLE {0} TYPE datetime;
",
            CHAT_TABLE,
        ))
        .await?;
    Ok(())
}

pub async fn create_chat(
    Json(mut chat): Json<ChatData>,
) -> Result<Json<Option<Chat>>, ServerError> {
    if chat.time.is_none() {
        chat.time = Some(Datetime::default())
    }
    let chat = CONN.create(CHAT_TABLE).content(chat).await?;

    Ok(Json(chat))
}

pub async fn get_chat(id: Path<String>) -> Result<Json<Option<Chat>>, ServerError> {
    let chat = CONN.select((CHAT_TABLE, &*id)).await?;
    Ok(Json(chat))
}

pub async fn update_chat(
    id: Path<String>,
    Json(mut chat): Json<ChatData>,
) -> Result<Json<Option<Chat>>, ServerError> {
    if chat.time.is_none() {
        chat.time = Some(Datetime::default())
    }
    let chat = CONN.update((CHAT_TABLE, &*id)).content(chat).await?;
    Ok(Json(chat))
}

pub async fn delete_chat(id: Path<String>) -> Result<Json<Option<Chat>>, ServerError> {
    let chat = CONN.delete((CHAT_TABLE, &*id)).await?;
    Ok(Json(chat))
}

pub async fn list_all_chats() -> Result<Json<Vec<Chat>>, ServerError> {
    let chats = CONN.select(CHAT_TABLE).await?;
    Ok(Json(chats))
}
