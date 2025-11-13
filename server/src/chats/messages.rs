use crate::chats::relationships::{
    MessageRelationship, RELATIONSHIP_TABLE, Reason, get_count_of_children,
};
use crate::files::FILE_TABLE;
use crate::generation::text::ChatQueryMessage;
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
    pub content: String,
    #[builder(default = "None")]
    pub model: Option<ModelData>,
    #[builder(default = "None")]
    pub thinking: Option<String>,
    #[serde(default = "Vec::new")]
    #[builder(default = "Vec::new()")]
    pub files: Vec<String>,
    #[builder(default = "None")]
    reason: Option<Reason>,
    #[builder(default = "None")]
    pub time: Option<Datetime>,
    #[serde(default = "Role::default")]
    #[builder(default = "Role::User")]
    pub role: Role,
}

impl Into<ChatQueryMessage> for Message {
    fn into(self) -> ChatQueryMessage {
        ChatQueryMessage {
            text: self.content,
            files: self.files,
            role: self.role,
        }
    }
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
    pub content: String,
    #[serde(default = "Vec::new")]
    pub files: Vec<String>,
    pub model: Option<ModelData>,
    pub thinking: Option<String>,
    pub role: Role,
    pub time: Datetime,
    pub id: RecordId,
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

pub async fn get_message_list_from_parent(
    id: Path<String>,
    Json(path): Json<Vec<i8>>,
) -> Result<Json<Vec<Message>>, ServerError> {
    let mut list = match read_message(Path(id.clone())).await?.0 {
        Some(x) => vec![x],
        _ => return Ok(Json(Vec::new())),
    };
    let mut parent: String = id.to_string();

    for index in path {
        let len = get_count_of_children(Path(parent.to_string())).await?.0;

        let index = if index < 0 {
            len - 1
        } else if index >= len as i8 {
            0
        } else {
            index as u8
        };

        let query: Vec<MessageRelationship> = CONN
            .query(&format!(
                "
                SELECT * FROM {0} WHERE parent = '{1}' and index = {2} ORDER BY index ASC LIMIT 1; 
            ",
                RELATIONSHIP_TABLE, &parent, index
            ))
            .await?
            .take(0)?;

        if query.is_empty() {
            break;
        } else {
            parent = query[0].child.to_string();
            if let Some(x) = read_message(Path(parent.clone())).await?.0 {
                list.push(x);
            } else {
                break;
            }
        }
    }

    let mut extra = get_default_message_list_from_parent(Path(parent)).await?.0;
    if extra.len() > 1 {
        let _ = extra.remove(0);
        list.append(&mut extra);
    }

    Ok(Json(list))
}

pub async fn get_default_message_list_from_parent(
    id: Path<String>,
) -> Result<Json<Vec<Message>>, ServerError> {
    let mut list = match read_message(Path(id.clone())).await?.0 {
        Some(x) => vec![x],
        _ => return Ok(Json(Vec::new())),
    };
    let mut parent: String = id.to_string();

    loop {
        let query: Vec<MessageRelationship> = CONN
            .query(&format!(
                "
                SELECT * FROM {0} WHERE parent = '{1}' ORDER BY index ASC LIMIT 1; 
            ",
                RELATIONSHIP_TABLE, &parent
            ))
            .await?
            .take(0)?;

        if query.is_empty() {
            break;
        } else {
            parent = query[0].child.to_string();
            if let Some(x) = read_message(Path(parent.clone())).await?.0 {
                list.push(x);
            } else {
                break;
            }
        }
    }

    Ok(Json(list))
}

pub async fn list_all_messages_from_parent(
    parent: Path<String>,
) -> Result<Json<Vec<Message>>, ServerError> {
    let query: Vec<MessageRelationship> = CONN
        .query(&format!(
            "
                SELECT * FROM {0} WHERE parent = '{1}' ORDER BY index ASC LIMIT 1; 
            ",
            RELATIONSHIP_TABLE, &*parent
        ))
        .await?
        .take(0)?;

    let mut messages = Vec::new();

    for relationship in query {
        if let Some(x) = read_message(Path(relationship.child.to_string())).await?.0 {
            messages.push(x)
        }
    }

    Ok(Json(messages))
}

pub async fn list_all_messages() -> Result<Json<Vec<Message>>, ServerError> {
    let chats = CONN.select(MESSAGE_TABLE).await?;
    Ok(Json(chats))
}
