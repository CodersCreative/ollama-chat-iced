pub mod messages;
pub mod previews;
pub mod relationships;

use axum::{Json, extract::Path};
use ochat_types::chats::{Chat, ChatData, previews::Preview};
use surrealdb::Datetime;

use crate::{CONN, chats::previews::PREVIEW_TABLE, errors::ServerError};

const CHAT_TABLE: &str = "chats";

pub async fn define_chats() -> Result<(), ServerError> {
    // Use a linking table for default_tools and chats
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS root ON TABLE {0} TYPE option<string>;
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

pub async fn set_chat_root(
    Path((id, root)): Path<(String, String)>,
) -> Result<Json<Option<Chat>>, ServerError> {
    Ok(Json(
        CONN.query(&format!(
            "UPDATE {0}:{1} SET root = '{2}';",
            CHAT_TABLE, &*id, &*root
        ))
        .await?
        .take(0)?,
    ))
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
    let _: Option<Preview> = CONN.delete((PREVIEW_TABLE, &*id)).await?;
    Ok(Json(chat))
}

pub async fn list_all_chats() -> Result<Json<Vec<Chat>>, ServerError> {
    let chats = CONN.select(CHAT_TABLE).await?;
    Ok(Json(chats))
}
