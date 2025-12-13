use crate::{CONN, errors::ServerError};
use axum::{Json, extract::Path};
use ochat_types::prompts::{Prompt, PromptData};
const PROMPTS_TABLE: &str = "prompts";

pub async fn define_prompts() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS title ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS command ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS content ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS downloads ON TABLE {0} TYPE option<int>;
DEFINE FIELD IF NOT EXISTS upvotes ON TABLE {0} TYPE option<int>;
DEFINE FIELD IF NOT EXISTS downvotes ON TABLE {0} TYPE option<int>;
DEFINE FIELD IF NOT EXISTS user ON TABLE {0} TYPE option<object>;

DEFINE ANALYZER prompts_analyzer TOKENIZERS class, blank FILTERS lowercase, ascii;
DEFINE INDEX title_index ON TABLE {0} COLUMNS title SEARCH ANALYZER prompts_analyzer BM25;
DEFINE INDEX command_index ON TABLE {0} COLUMNS command SEARCH ANALYZER prompts_analyzer BM25;
DEFINE INDEX content_index ON TABLE {0} COLUMNS content SEARCH ANALYZER prompts_analyzer BM25;
",
            PROMPTS_TABLE,
        ))
        .await?;
    Ok(())
}

pub async fn add_prompt(
    Json(prompts): Json<PromptData>,
) -> Result<Json<Option<Prompt>>, ServerError> {
    Ok(Json(CONN.create(PROMPTS_TABLE).content(prompts).await?))
}

pub async fn update_prompt(
    id: Path<String>,
    Json(prompts): Json<PromptData>,
) -> Result<Json<Option<Prompt>>, ServerError> {
    Ok(Json(
        CONN.update((PROMPTS_TABLE, id.trim()))
            .content(prompts)
            .await?,
    ))
}

pub async fn get_prompt(id: Path<String>) -> Result<Json<Option<Prompt>>, ServerError> {
    Ok(Json(CONN.select((PROMPTS_TABLE, id.trim())).await?))
}

pub async fn search_prompts(search: Path<String>) -> Result<Json<Vec<Prompt>>, ServerError> {
    Ok(Json(CONN
        .query(&format!(
            "            
SELECT *, search::score(1) + search::score(2) + search::score(3) AS score FROM {0} WHERE title @1@ {1} or command @2@ {1} or content @3@ {1} ORDER BY score DESC;
",
            PROMPTS_TABLE, search.trim()
        ))
        .await?
        .take(0)?))
}

pub async fn delete_prompt(id: Path<String>) -> Result<Json<Option<Prompt>>, ServerError> {
    Ok(Json(CONN.delete((PROMPTS_TABLE, id.trim())).await?))
}

pub async fn list_all_prompts() -> Result<Json<Vec<Prompt>>, ServerError> {
    Ok(Json(CONN.select(PROMPTS_TABLE).await?))
}
