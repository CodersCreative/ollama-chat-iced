pub mod relationships;

use axum::{Json, extract::Path};
use ochat_types::options::{GenOptions, GenOptionsData};

use crate::{CONN, errors::ServerError};

pub const GEN_OPTIONS_TABLE: &str = "gen_options";

pub async fn define_gen_options() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS name ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS data ON TABLE {0} TYPE array<string, 16>;

DEFINE ANALYZER options_analyzer TOKENIZERS class, blank FILTERS lowercase, ascii;
DEFINE INDEX name_index ON TABLE {0} COLUMNS name SEARCH ANALYZER options_analyzer BM25;
",
            GEN_OPTIONS_TABLE,
        ))
        .await?;
    Ok(())
}

pub async fn add_gen_options(
    Json(options): Json<GenOptionsData>,
) -> Result<Json<Option<GenOptions>>, ServerError> {
    Ok(Json(CONN.create(GEN_OPTIONS_TABLE).content(options).await?))
}

pub async fn update_gen_options(
    id: Path<String>,
    Json(options): Json<GenOptionsData>,
) -> Result<Json<Option<GenOptions>>, ServerError> {
    Ok(Json(
        CONN.update((GEN_OPTIONS_TABLE, &*id))
            .content(options)
            .await?,
    ))
}

pub async fn get_gen_options(id: Path<String>) -> Result<Json<Option<GenOptions>>, ServerError> {
    Ok(Json(CONN.select((GEN_OPTIONS_TABLE, &*id)).await?))
}

pub async fn search_gen_options(
    search: Path<String>,
) -> Result<Json<Vec<GenOptions>>, ServerError> {
    Ok(Json(
        CONN.query(&format!(
            "            
SELECT *, search::score(1) AS score FROM {0} WHERE name @1@ {1} ORDER BY score DESC LIMIT 25;
",
            GEN_OPTIONS_TABLE, &*search
        ))
        .await?
        .take(0)?,
    ))
}

pub async fn delete_gen_options(id: Path<String>) -> Result<Json<Option<GenOptions>>, ServerError> {
    Ok(Json(CONN.delete((GEN_OPTIONS_TABLE, &*id)).await?))
}

pub async fn list_all_gen_options() -> Result<Json<Vec<GenOptions>>, ServerError> {
    Ok(Json(CONN.select(GEN_OPTIONS_TABLE).await?))
}
