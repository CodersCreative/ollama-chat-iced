pub mod messages;
pub mod previews;
pub mod route;

use crate::backend::{
    CONN,
    chats::{
        messages::{create_message, create_message_with_parent},
        previews::PREVIEW_TABLE,
    },
    errors::ServerError,
};
use axum::{Json, extract::Path};
use ochat_types::chats::{Chat, ChatData, messages::MessageData, previews::Preview};

const CHAT_TABLE: &str = "chats";

pub async fn define_chats() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMAFULL
    PERMISSIONS FOR select, update, delete WHERE user_id = record::id($auth.id) FOR create FULL;
DEFINE FIELD IF NOT EXISTS user_id ON TABLE {0} TYPE string DEFAULT ALWAYS record::id($auth.id);
DEFINE FIELD IF NOT EXISTS root ON TABLE {0} TYPE option<string>;
DEFINE FIELD IF NOT EXISTS time ON TABLE {0} TYPE string DEFAULT <string>time::now();
",
            CHAT_TABLE,
        ))
        .await?;
    Ok(())
}

pub async fn branch_new_chat(
    Json(mut messages): Json<Vec<MessageData>>,
) -> Result<Json<Option<Chat>>, ServerError> {
    messages.iter_mut().for_each(|x| {
        x.children.clear();
    });

    let root = create_message(Json(messages.remove(0))).await?.0.unwrap();
    let mut parent = root.id.key().to_string();

    let chat = create_chat(Json(ChatData {
        user_id: None,
        root: Some(parent.clone()),
        time: None,
    }))
    .await?;

    for message in messages {
        if let Some(x) = create_message_with_parent(Path(parent.clone()), Json(message))
            .await?
            .0
        {
            parent = x.id.key().to_string();
        }
    }

    Ok(chat)
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
