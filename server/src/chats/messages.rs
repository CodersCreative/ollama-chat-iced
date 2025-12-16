use crate::chats::relationships::{RELATIONSHIP_TABLE, get_count_of_children};
use crate::{CONN, errors::ServerError};
use axum::{Json, extract::Path};
use ochat_types::chats::messages::{Message, MessageCanChange, MessageData};
use ochat_types::chats::relationships::{MessageRelationship, MessageRelationshipDataBuilder};
use ochat_types::surreal::Datetime;

const MESSAGE_TABLE: &str = "messages";

// Change time to use surreal datetime
pub async fn define_messages() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS content ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS thinking ON TABLE {0} TYPE option<string>;
DEFINE FIELD IF NOT EXISTS model ON TABLE {0} TYPE option<object>;
DEFINE FIELD IF NOT EXISTS role ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS time ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS files ON TABLE {0} TYPE array<string>;
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
    let reason = chat.reason.clone();
    let chat: Option<Message> = CONN.create(MESSAGE_TABLE).content(chat).await?;

    if let Some(chat) = &chat {
        let _ = super::relationships::create_message_relationship(Json(
            MessageRelationshipDataBuilder::default()
                .parent(parent.trim().to_string())
                .child(chat.id.key().to_string())
                .reason(reason)
                .build()
                .unwrap(),
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

pub async fn delete_message(id: Path<String>) -> Result<Json<Option<Message>>, ServerError> {
    Ok(Json(CONN.delete((MESSAGE_TABLE, id.trim())).await?))
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
        let len = get_count_of_children(Path(parent.to_string())).await?.0 as i8;

        let mut index = index;

        while index < 0 {
            index += len;
        }

        while index > len {
            index -= len;
        }

        let query: Vec<MessageRelationship> = CONN
            .query(&format!(
                "
                SELECT * FROM {0} WHERE parent = '{1}' and index = {2} ORDER BY index ASC LIMIT 1; 
            ",
                RELATIONSHIP_TABLE,
                parent.trim(),
                index
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

pub async fn get_can_change(id: Path<String>) -> Result<Json<MessageCanChange>, ServerError> {
    let mut parent: Vec<MessageRelationship> = CONN
        .query(&format!(
            "
                SELECT * FROM {0} WHERE child = '{1}';
            ",
            RELATIONSHIP_TABLE,
            id.trim(),
        ))
        .await?
        .take(0)?;

    if parent.is_empty() {
        return Ok(Json(MessageCanChange {
            id: id.to_string(),
            can_change: false,
        }));
    }

    let parent = parent.remove(0).parent;

    let len = get_count_of_children(Path(parent)).await?.0 as i8;

    Ok(Json(MessageCanChange {
        id: id.to_string(),
        can_change: if len > 1 { true } else { false },
    }))
}

pub async fn get_can_change_list_from_parent(
    id: Path<String>,
    Json(path): Json<Vec<i8>>,
) -> Result<Json<Vec<MessageCanChange>>, ServerError> {
    let mut list = vec![MessageCanChange {
        id: id.to_string(),
        can_change: false,
    }];
    let mut parent: String = id.to_string();

    for index in path {
        let len = get_count_of_children(Path(parent.to_string())).await?.0 as i8;

        let mut index = index;

        while index < 0 {
            index += len;
        }

        while index > len {
            index -= len;
        }

        let query: Vec<MessageRelationship> = CONN
            .query(&format!(
                "
                SELECT * FROM {0} WHERE parent = '{1}' and index = {2} ORDER BY index ASC LIMIT 1; 
            ",
                RELATIONSHIP_TABLE,
                parent.trim(),
                index
            ))
            .await?
            .take(0)?;

        if query.is_empty() {
            break;
        } else {
            parent = query[0].child.to_string();

            list.push(MessageCanChange {
                id: parent.clone(),
                can_change: if len > 1 { true } else { false },
            });
        }
    }

    let mut extra = get_default_can_change_list_from_parent(Path(parent))
        .await?
        .0;
    if extra.len() > 1 {
        let _ = extra.remove(0);
        list.append(&mut extra);
    }

    Ok(Json(list))
}

pub async fn get_default_can_change_list_from_parent(
    id: Path<String>,
) -> Result<Json<Vec<MessageCanChange>>, ServerError> {
    let mut list = vec![MessageCanChange {
        id: id.to_string(),
        can_change: false,
    }];
    let mut parent: String = id.to_string();

    loop {
        let len = get_count_of_children(Path(parent.to_string())).await?.0 as i8;

        let query: Vec<MessageRelationship> = CONN
            .query(&format!(
                "
                SELECT * FROM {0} WHERE parent = '{1}' ORDER BY index ASC LIMIT 1; 
            ",
                RELATIONSHIP_TABLE,
                parent.trim(),
            ))
            .await?
            .take(0)?;

        if query.is_empty() {
            break;
        } else {
            parent = query[0].child.to_string();

            list.push(MessageCanChange {
                id: parent.clone(),
                can_change: if len > 1 { true } else { false },
            });
        }
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
    let mut parent: String = id.trim().to_string();

    loop {
        let query: Vec<MessageRelationship> = CONN
            .query(&format!(
                "
                SELECT * FROM {0} WHERE parent = '{1}' ORDER BY index ASC LIMIT 1; 
            ",
                RELATIONSHIP_TABLE,
                parent.trim()
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
            RELATIONSHIP_TABLE,
            parent.trim()
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
    Ok(Json(CONN.select(MESSAGE_TABLE).await?))
}
