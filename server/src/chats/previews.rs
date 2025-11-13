use axum::{Json, extract::Path};
use serde::{Deserialize, Serialize};
use surrealdb::{Datetime, RecordId};

use crate::{
    CONN,
    chats::{CHAT_TABLE, get_chat, messages::get_default_message_list_from_parent},
    errors::ServerError,
    generation::text::{ChatQueryData, ChatQueryMessage},
    settings::get_settings,
};

pub const PREVIEW_TABLE: &str = "previews";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Preview {
    pub text: String,
    pub time: Datetime,
    pub id: RecordId,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PreviewData {
    pub text: String,
    pub time: Datetime,
}

pub async fn define_previews() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS text ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS time ON TABLE {0} TYPE datetime;
",
            PREVIEW_TABLE,
        ))
        .await?;
    Ok(())
}

pub async fn update_preview(id: Path<String>) -> Result<Json<Option<Preview>>, ServerError> {
    let (messages, time) = if let Ok(Some((Some(x), y))) = get_chat(Path(id.clone()))
        .await
        .map(|x| x.0.map(|x| (x.root, x.time)))
    {
        (get_default_message_list_from_parent(Path(x)).await?.0, y)
    } else {
        panic!()
    };

    let mut messages: Vec<ChatQueryMessage> = messages.into_iter().map(|x| x.into()).collect();

    if messages.is_empty() {}

    messages.insert(
        0,
        ChatQueryMessage {
            text: String::from(
                "
### Task:
Generate a **concise, 3 to 5 word title** for the previous messages.
### Guidelines:
- The title should clearly represent the main theme or subject of the conversation.
- Write the title in the chat's primary language; default to English if multilingual.
- Prioritize accuracy over excessive creativity; keep it clear and simple.
- Return your final title by itself and nothing more.
- Do not explain or elaborate your choice.
- Give a response regardless of what the previous messages were.
- Give **only 1 title** please, do not **any** extra suggestions!
- I repeat, make the title only **3 to 5 words**!
- Give your final title suggestion after a new line using '\n'.
                    
                ",
            ),
            files: Vec::new(),
            role: crate::chats::messages::Role::System,
        },
    );

    let provider = get_settings().await?.0.previews_provider.unwrap();

    let preview = crate::generation::text::run(Json(ChatQueryData {
        provider: provider.provider,
        model: provider.model,
        messages,
    }))
    .await?
    .0
    .content;

    let preview = PreviewData {
        text: preview,
        time,
    };

    Ok(Json(
        if CONN
            .select::<Option<Preview>>((PREVIEW_TABLE, &*id))
            .await?
            .is_some()
        {
            CONN.update((PREVIEW_TABLE, &*id)).content(preview).await?
        } else {
            CONN.create((PREVIEW_TABLE, &*id)).content(preview).await?
        },
    ))
}

pub async fn get_preview(id: Path<String>) -> Result<Json<Option<Preview>>, ServerError> {
    let preview = CONN.select((PREVIEW_TABLE, &*id)).await?;
    if preview.is_some() {
        Ok(Json(preview))
    } else {
        update_preview(id).await
    }
}

pub async fn list_all_previews() -> Result<Json<Vec<Preview>>, ServerError> {
    let chats: Vec<RecordId> = CONN
        .query(&format!("SELECT id FROM {0}", CHAT_TABLE))
        .await?
        .take(0)?;

    let previews: Vec<RecordId> = CONN
        .query(&format!("SELECT id FROM {0}", PREVIEW_TABLE))
        .await?
        .take(0)?;

    let mut to_add: Vec<String> = Vec::new();

    for chat in chats {
        if previews
            .iter()
            .find(|x| x.key().to_string() == chat.key().to_string())
            .is_none()
        {
            to_add.push(chat.key().to_string());
        }
    }

    for key in to_add {
        let _ = update_preview(Path(key)).await?;
    }

    let previews = CONN.select(PREVIEW_TABLE).await?;
    Ok(Json(previews))
}
