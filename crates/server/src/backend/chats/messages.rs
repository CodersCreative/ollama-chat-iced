use crate::backend::{CONN, errors::ServerError};
use async_recursion::async_recursion;
use axum::{Json, extract::Path};
use ochat_types::chats::messages::{Message, MessageData};
use ochat_types::surreal::Datetime;

const MESSAGE_TABLE: &str = "messages";

// Change time to use surreal datetime
pub async fn define_messages() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS content ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS thinking ON TABLE {0} TYPE option<string>;
DEFINE FIELD IF NOT EXISTS model ON TABLE {0} TYPE option<object>;
DEFINE FIELD IF NOT EXISTS role ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS time ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS files ON TABLE {0} TYPE array<string>;
DEFINE FIELD IF NOT EXISTS children ON TABLE {0} TYPE array<string>;
",
            MESSAGE_TABLE
        ))
        .await?;
    Ok(())
}

pub async fn create_message(
    Json(mut chat): Json<MessageData>,
) -> Result<Json<Option<Message>>, ServerError> {
    if chat.time.is_none() {
        chat.time = Some(Datetime::default());
    }
    let chat: Option<Message> = CONN.create(MESSAGE_TABLE).content(chat).await?;
    Ok(Json(chat))
}

pub async fn create_message_with_parent(
    parent: Path<String>,
    Json(mut chat): Json<MessageData>,
) -> Result<Json<Option<Message>>, ServerError> {
    if chat.time.is_none() {
        chat.time = Some(Datetime::default());
    }
    let chat: Option<Message> = CONN.create(MESSAGE_TABLE).content(chat).await?;

    if let Some(chat) = &chat {
        CONN.query(&format!(
            "UPDATE {0}:{1} SET children += '{2}';",
            MESSAGE_TABLE,
            parent.trim(),
            chat.id.key().to_string().trim(),
        ))
        .await?;
    }

    Ok(Json(chat))
}

pub async fn read_message(id: Path<String>) -> Result<Json<Option<Message>>, ServerError> {
    Ok(Json(CONN.select((MESSAGE_TABLE, id.trim())).await?))
}

pub async fn update_message(
    id: Path<String>,
    Json(mut chat): Json<MessageData>,
) -> Result<Json<Option<Message>>, ServerError> {
    if chat.time.is_none() {
        chat.time = Some(Datetime::default());
    }
    let chat: Option<Message> = CONN
        .update((MESSAGE_TABLE, id.trim()))
        .content(chat)
        .await?;

    Ok(Json(chat))
}

pub async fn list_all_messages() -> Result<Json<Vec<Message>>, ServerError> {
    Ok(Json(CONN.select(MESSAGE_TABLE).await?))
}

pub async fn delete_message(id: Path<String>) -> Result<Json<Option<Message>>, ServerError> {
    Ok(Json(CONN.delete((MESSAGE_TABLE, id.trim())).await?))
}

pub async fn list_all_messages_from_parent(
    parent: Path<String>,
) -> Result<Json<Vec<Message>>, ServerError> {
    let parent = read_message(parent).await?.0.unwrap();
    let mut messages = Vec::new();

    for child in parent.children {
        if let Some(x) = read_message(Path(child.clone())).await?.0 {
            messages.push(x)
        }
    }

    Ok(Json(messages))
}

#[async_recursion]
pub async fn get_all_messages_from_root(
    id: Path<String>,
) -> Result<Json<Vec<Message>>, ServerError> {
    let mut messages = list_all_messages_from_parent(Path(id.clone())).await?.0;
    let mut fin: Vec<Message> = Vec::new();
    for message in messages.iter().filter(|x| x.id.key().to_string() != id.0) {
        let mut extra = get_all_messages_from_root(Path(message.id.key().to_string()))
            .await?
            .0;
        fin.append(&mut extra);
    }

    fin.append(&mut messages);
    Ok(Json(fin))
}

pub async fn get_default_message_list_from_parent(
    id: Path<String>,
) -> Result<Json<Vec<Message>>, ServerError> {
    let mut list = match read_message(Path(id.clone())).await?.0 {
        Some(x) => vec![x],
        _ => return Ok(Json(Vec::new())),
    };

    loop {
        if list.last().unwrap().children.is_empty() {
            break;
        }

        if let Some(x) = read_message(Path(list.last().unwrap().children.first().unwrap().clone()))
            .await?
            .0
        {
            if list.contains(&x) {
                break;
            }
            list.push(x);
        } else {
            break;
        }
    }

    Ok(Json(list))
}
