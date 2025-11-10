use axum::{Json, extract::Path};
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

use crate::{CONN, errors::ServerError};

const CHAT_MESSAGE_TABLE: &str = "chats";
const VIDEOS_TABLE: &str = "videos";
const IMAGES_TABLE: &str = "images";
const AUDIO_TABLE: &str = "audios";
const FILE_TABLE: &str = "files";

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessageData {
    name: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChatMessage {
    name: String,
    id: RecordId,
}

pub async fn define_chat() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS
DEFINE FIELD IF NOT EXISTS content ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS thinking ON TABLE {0} TYPE option<string>;
DEFINE FIELD IF NOT EXISTS role ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS unix ON TABLE {0} TYPE datetime;

DEFINE TABLE IF NOT EXISTS {1} SCHEMALESS
DEFINE FIELD IF NOT EXISTS content ON TABLE {1} TYPE string;

DEFINE TABLE IF NOT EXISTS {2} SCHEMALESS
DEFINE FIELD IF NOT EXISTS content ON TABLE {2} TYPE string;

DEFINE TABLE IF NOT EXISTS {3} SCHEMALESS
DEFINE FIELD IF NOT EXISTS  ON TABLE {3} TYPE string;

DEFINE TABLE IF NOT EXISTS {4} SCHEMALESS
DEFINE FIELD IF NOT EXISTS  ON TABLE {4} TYPE string;
",
            CHAT_MESSAGE_TABLE, VIDEOS_TABLE, IMAGES_TABLE, AUDIO_TABLE, FILE_TABLE
        ))
        .await?;
    Ok(())
}

pub async fn create_chat_message(
    id: Path<String>,
    Json(chat): Json<ChatMessageData>,
) -> Result<Json<Option<ChatMessage>>, ServerError> {
    let chat = CONN
        .create((CHAT_MESSAGE_TABLE, &*id))
        .content(chat)
        .await?;

    Ok(Json(chat))
}

pub async fn read_chat_message(id: Json<String>) -> Result<Json<Option<ChatMessage>>, ServerError> {
    let chat = CONN.select((CHAT_MESSAGE_TABLE, &*id)).await?;
    Ok(Json(chat))
}

pub async fn update_chat_message(
    id: Path<String>,
    Json(chat): Json<ChatMessageData>,
) -> Result<Json<Option<ChatMessage>>, ServerError> {
    let chat = CONN
        .update((CHAT_MESSAGE_TABLE, &*id))
        .content(chat)
        .await?;
    Ok(Json(chat))
}

pub async fn delete_chat_message(id: String) -> Result<Json<Option<ChatMessage>>, ServerError> {
    let person = CONN.delete((CHAT_MESSAGE_TABLE, &*id)).await?;
    Ok(Json(person))
}

pub async fn list_all_chat_messages() -> Result<Json<Vec<ChatMessage>>, ServerError> {
    let people = CONN.select(CHAT_MESSAGE_TABLE).await?;
    Ok(Json(people))
}
