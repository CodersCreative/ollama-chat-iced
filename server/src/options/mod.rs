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

DEFINE ANALYZER IF NOT EXISTS analyzer TOKENIZERS blank FILTERS lowercase, snowball(english);
DEFINE INDEX IF NOT EXISTS name_index ON {0} FIELDS text SEARCH ANALYZER analyzer BM25;
",
            GEN_OPTIONS_TABLE,
        ))
        .await?;
    Ok(())
}

pub async fn add_gen_options(
    Json(options): Json<GenOptionsData>,
) -> Result<Json<Option<GenOptions>>, ServerError> {
    let options: Option<GenOptions> = CONN.create(GEN_OPTIONS_TABLE).content(options).await?;

    Ok(Json(options))
}

pub async fn update_gen_options(
    id: Path<String>,
    Json(options): Json<GenOptionsData>,
) -> Result<Json<Option<GenOptions>>, ServerError> {
    let options = CONN
        .update((GEN_OPTIONS_TABLE, &*id))
        .content(options)
        .await?;
    Ok(Json(options))
}

pub async fn get_gen_options(id: Path<String>) -> Result<Json<Option<GenOptions>>, ServerError> {
    let options = CONN.select((GEN_OPTIONS_TABLE, &*id)).await?;
    Ok(Json(options))
}

pub async fn search_gen_options(
    search: Path<String>,
) -> Result<Json<Vec<GenOptions>>, ServerError> {
    let result = CONN
        .query(&format!(
            "            
SELECT *, search::score(1) AS score FROM {0} WHERE name @1@ {1} ORDER BY score DESC LIMIT 25;
",
            GEN_OPTIONS_TABLE, &*search
        ))
        .await?
        .take(0)?;

    Ok(Json(result))
}

pub async fn delete_gen_options(id: Path<String>) -> Result<Json<Option<GenOptions>>, ServerError> {
    let options = CONN.delete((GEN_OPTIONS_TABLE, &*id)).await?;
    Ok(Json(options))
}

pub async fn list_all_gen_options() -> Result<Json<Vec<GenOptions>>, ServerError> {
    let options = CONN.select(GEN_OPTIONS_TABLE).await?;
    Ok(Json(options))
}
