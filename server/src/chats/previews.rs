use axum::{Json, extract::Path};
use ochat_types::{
    chats::{
        messages::Role,
        previews::{Preview, PreviewData},
    },
    generation::text::{ChatQueryData, ChatQueryMessage},
};

use crate::{
    CONN,
    chats::{get_chat, messages::get_default_message_list_from_parent},
    errors::ServerError,
    settings::get_settings,
};

pub const PREVIEW_TABLE: &str = "previews";

pub async fn define_previews() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS text ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS time ON TABLE {0} TYPE datetime;

DEFINE ANALYZER IF NOT EXISTS analyzer TOKENIZERS blank FILTERS lowercase, snowball(english);
DEFINE INDEX IF NOT EXISTS text_index ON {0} FIELDS text SEARCH ANALYZER analyzer BM25;
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
            role: Role::System,
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

pub async fn search_previews(search: Path<String>) -> Result<Json<Vec<Preview>>, ServerError> {
    let result = CONN
        .query(&format!(
            "            
SELECT *, search::score(1) AS score FROM {0} WHERE title @1@ {1} ORDER BY score DESC LIMIT 25;
",
            PREVIEW_TABLE, &*search
        ))
        .await?
        .take(0)?;

    Ok(Json(result))
}

pub async fn list_all_previews() -> Result<Json<Vec<Preview>>, ServerError> {
    let previews = CONN.select(PREVIEW_TABLE).await?;
    Ok(Json(previews))
}
