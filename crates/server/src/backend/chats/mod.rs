pub mod messages;
pub mod previews;
pub mod relationships;
pub mod route;

use crate::backend::{CONN, chats::previews::PREVIEW_TABLE, errors::ServerError};
use axum::{Json, extract::Path};
use ochat_types::chats::{Chat, ChatData, previews::Preview};

const CHAT_TABLE: &str = "chats";

pub async fn define_chats() -> Result<(), ServerError> {
    // Use a linking table for default_tools and chats
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMAFULL
    PERMISSIONS FOR select, update, delete WHERE user_id = $auth.id FOR create FULL;
DEFINE FIELD IF NOT EXISTS user_id ON TABLE {0} TYPE record DEFAULT ALWAYS $auth.id;
DEFINE FIELD IF NOT EXISTS root ON TABLE {0} TYPE option<string>;
DEFINE FIELD IF NOT EXISTS time ON TABLE {0} TYPE string DEFAULT <string>time::now();
",
            CHAT_TABLE,
        ))
        .await?;
    Ok(())
}

pub async fn create_chat(Json(chat): Json<ChatData>) -> Result<Json<Option<Chat>>, ServerError> {
    Ok(Json(CONN.create(CHAT_TABLE).content(chat).await?))
}

pub async fn get_chat(id: Path<String>) -> Result<Json<Option<Chat>>, ServerError> {
    Ok(Json(CONN.select((CHAT_TABLE, id.trim())).await?))
}

pub async fn set_chat_root(
    Path((id, root)): Path<(String, String)>,
) -> Result<Json<Option<Chat>>, ServerError> {
    Ok(Json(
        CONN.query(&format!(
            "UPDATE {0}:{1} SET root = '{2}';",
            CHAT_TABLE,
            id.trim(),
            root.trim()
        ))
        .await?
        .take(0)?,
    ))
}

pub async fn update_chat(
    id: Path<String>,
    Json(chat): Json<ChatData>,
) -> Result<Json<Option<Chat>>, ServerError> {
    Ok(Json(
        CONN.update((CHAT_TABLE, id.trim())).content(chat).await?,
    ))
}

pub async fn delete_chat(id: Path<String>) -> Result<Json<Option<Chat>>, ServerError> {
    let _: Option<Preview> = CONN.delete((PREVIEW_TABLE, &*id)).await?;
    Ok(Json(CONN.delete((CHAT_TABLE, id.trim())).await?))
}
pub async fn list_all_chats() -> Result<Json<Vec<Chat>>, ServerError> {
    Ok(Json(CONN.select(CHAT_TABLE).await?))
}
