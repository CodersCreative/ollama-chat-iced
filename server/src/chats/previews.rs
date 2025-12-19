use axum::{Extension, Json, extract::Path};
use ochat_types::{
    chats::{
        messages::Role,
        previews::{Preview, PreviewData},
    },
    generation::text::{ChatQueryData, ChatQueryMessage},
    surreal::Datetime,
    user::User,
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
DEFINE FIELD IF NOT EXISTS user_id ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS text ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS time ON TABLE {0} TYPE string;

DEFINE ANALYZER previews_analyzer TOKENIZERS class, blank FILTERS lowercase, ascii;
DEFINE INDEX text_index ON TABLE {0} COLUMNS text SEARCH ANALYZER previews_analyzer BM25;
",
            PREVIEW_TABLE,
        ))
        .await?;
    Ok(())
}

pub async fn update_preview(
    Extension(user): Extension<User>,
    id: Path<String>,
) -> Result<Json<Option<Preview>>, ServerError> {
    let (messages, time) = match get_chat(Path(id.clone())).await.map(|x| x.0) {
        Ok(Some(chat)) if chat.root.is_some() => (
            get_default_message_list_from_parent(Path(chat.root.unwrap()))
                .await?
                .0,
            chat.time,
        ),
        Ok(Some(chat)) => (Vec::new(), chat.time),
        Ok(None) => (Vec::new(), Datetime::default()),
        _ => panic!(),
    };

    if messages.is_empty() {
        let preview = PreviewData {
            user_id: Some(user.id.key().to_string()),
            text: String::from("New Chat"),
            time: time.clone(),
        };
        return Ok(Json(
            if CONN
                .select::<Option<Preview>>((PREVIEW_TABLE, id.trim()))
                .await?
                .is_some()
            {
                CONN.update((PREVIEW_TABLE, id.trim()))
                    .content(preview)
                    .await?
            } else {
                CONN.create((PREVIEW_TABLE, id.trim()))
                    .content(preview)
                    .await?
            },
        ));
    }

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
        user_id: Some(user.id.key().to_string()),
        text: preview,
        time,
    };

    Ok(Json(
        if CONN
            .select::<Option<Preview>>((PREVIEW_TABLE, id.trim()))
            .await?
            .is_some()
        {
            CONN.update((PREVIEW_TABLE, id.trim()))
                .content(preview)
                .await?
        } else {
            CONN.create((PREVIEW_TABLE, id.trim()))
                .content(preview)
                .await?
        },
    ))
}

pub async fn get_preview(
    user: Extension<User>,
    id: Path<String>,
) -> Result<Json<Option<Preview>>, ServerError> {
    let preview = CONN.select((PREVIEW_TABLE, id.trim())).await?;
    if preview.is_some() {
        Ok(Json(preview))
    } else {
        update_preview(user, id).await
    }
}

pub async fn search_previews(search: Path<String>) -> Result<Json<Vec<Preview>>, ServerError> {
    Ok(Json(
        CONN.query(&format!(
            "            
SELECT *, search::score(1) AS score FROM {0} WHERE text @1@ '{1}' ORDER BY score DESC;
",
            PREVIEW_TABLE,
            search.trim()
        ))
        .await?
        .take(0)?,
    ))
}

pub async fn list_all_previews() -> Result<Json<Vec<Preview>>, ServerError> {
    Ok(Json(CONN.select(PREVIEW_TABLE).await?))
}
