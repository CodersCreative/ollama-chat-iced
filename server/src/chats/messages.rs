use crate::chats::relationships::Reason;
use crate::files::FILE_TABLE;
use crate::{CONN, errors::ServerError};
use axum::{Json, extract::Path};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use surrealdb::{Datetime, RecordId};

const MESSAGE_TABLE: &str = "messages";
const MESSAGE_FILE_TABLE: &str = "message_files";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Role {
    #[default]
    User,
    AI,
    Function,
    System,
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct MessageData {
    content: String,
    #[builder(default = "None")]
    model: Option<ModelData>,
    #[builder(default = "None")]
    thinking: Option<String>,
    #[serde(default = "Vec::new")]
    #[builder(default = "Vec::new()")]
    files: Vec<String>,
    #[builder(default = "None")]
    reason: Option<Reason>,
    #[builder(default = "None")]
    time: Option<Datetime>,
    #[serde(default = "Role::default")]
    #[builder(default = "Role::User")]
    role: Role,
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
struct StoredMessageData {
    content: String,
    model: Option<ModelData>,
    thinking: Option<String>,
    time: Datetime,
    role: Role,
}

impl From<MessageData> for StoredMessageData {
    fn from(value: MessageData) -> Self {
        Self {
            content: value.content,
            model: value.model,
            thinking: value.thinking,
            time: match value.time {
                Some(x) => x,
                None => Datetime::default(),
            },
            role: value.role,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct ModelData {
    provider: String,
    model: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    content: String,
    #[serde(default = "Vec::new")]
    files: Vec<String>,
    model: Option<ModelData>,
    thinking: Option<String>,
    role: Role,
    time: Datetime,
    id: RecordId,
}

pub async fn define_messages() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS content ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS thinking ON TABLE {0} TYPE option<string>;
DEFINE FIELD IF NOT EXISTS role ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS time ON TABLE {0} TYPE datetime;
",
            MESSAGE_TABLE
        ))
        .await?;
    Ok(())
}

pub async fn create_message(
    Json(chat): Json<MessageData>,
) -> Result<Json<Option<Message>>, ServerError> {
    let files = chat.files.clone();
    let data = StoredMessageData::from(chat);
    let chat: Option<Message> = CONN.create(MESSAGE_TABLE).content(data).await?;

    if let Some(chat) = &chat {
        for file in files.into_iter() {
            let _ = CONN
                .query(&format!(
                    "RELATE {MESSAGE_TABLE}:{0} -> {MESSAGE_FILE_TABLE} -> {FILE_TABLE}:{file};",
                    chat.id.key()
                ))
                .await?;
        }
    }

    Ok(Json(chat))
}

pub async fn create_message_with_parent(
    parent: Path<String>,
    Json(chat): Json<MessageData>,
) -> Result<Json<Option<Message>>, ServerError> {
    let files = chat.files.clone();
    let reason = chat.reason.clone();
    let data = StoredMessageData::from(chat);
    let chat: Option<Message> = CONN.create(MESSAGE_TABLE).content(data).await?;

    if let Some(chat) = &chat {
        let _ = super::relationships::create_message_relationship(Json(
            super::relationships::MessageRelationshipDataBuilder::default()
                .parent(parent.to_string())
                .child(chat.id.key().to_string())
                .reason(reason)
                .build()
                .unwrap(),
        ))
        .await?;

        for file in files.into_iter() {
            let _ = CONN
                .query(&format!(
                    "RELATE {MESSAGE_TABLE}:{0} -> {MESSAGE_FILE_TABLE} -> {FILE_TABLE}:{file};",
                    chat.id.key()
                ))
                .await?;
        }
    }

    Ok(Json(chat))
}

pub async fn read_message(id: Path<String>) -> Result<Json<Option<Message>>, ServerError> {
    let chat = CONN.select((MESSAGE_TABLE, &*id)).await?;
    Ok(Json(chat))
}

pub async fn update_message(
    id: Path<String>,
    Json(chat): Json<MessageData>,
) -> Result<Json<Option<Message>>, ServerError> {
    let files = chat.files.clone();
    let data = StoredMessageData::from(chat);
    let chat: Option<Message> = CONN.update((MESSAGE_TABLE, &*id)).content(data).await?;

    if let Some(chat) = &chat {
        let _ = CONN.query(&format!(
            "DELETE {0} WHERE in = {1}",
            MESSAGE_FILE_TABLE,
            chat.id.key()
        ));
        for file in files.into_iter() {
            let _ = CONN
                .query(&format!(
                    "RELATE {MESSAGE_TABLE}:{0} -> {MESSAGE_FILE_TABLE} -> {FILE_TABLE}:{file};",
                    chat.id.key()
                ))
                .await?;
        }
    }

    Ok(Json(chat))
}

pub async fn delete_message(id: Path<String>) -> Result<Json<Option<Message>>, ServerError> {
    let chat = CONN.delete((MESSAGE_TABLE, &*id)).await?;
    Ok(Json(chat))
}

pub async fn list_all_messages() -> Result<Json<Vec<Message>>, ServerError> {
    let chats = CONN.select(MESSAGE_TABLE).await?;
    Ok(Json(chats))
}
