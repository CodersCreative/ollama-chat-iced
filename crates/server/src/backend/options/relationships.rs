use crate::backend::{CONN, errors::ServerError};
use axum::{Json, extract::Path};
use ochat_types::{
    options::{
        GenOptions,
        relationships::{GenModelRelationship, GenModelRelationshipData},
    },
    settings::SettingsProvider,
};

pub const GEN_MODELS_TABLE: &str = "gen_models";

pub async fn define_gen_models() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMAFULL
    PERMISSIONS FOR select, update, delete WHERE user_id = $auth.id FOR create FULL;
DEFINE FIELD IF NOT EXISTS user_id ON TABLE {0} TYPE record DEFAULT ALWAYS $auth.id;
DEFINE FIELD IF NOT EXISTS provider ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS model ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS option ON TABLE {0} TYPE string;
",
            GEN_MODELS_TABLE,
        ))
        .await?;
    Ok(())
}

pub async fn add_gen_models(
    Json(relationship): Json<GenModelRelationshipData>,
) -> Result<Json<Option<GenModelRelationship>>, ServerError> {
    Ok(Json(
        CONN.create(GEN_MODELS_TABLE).content(relationship).await?,
    ))
}

pub async fn update_gen_models(
    id: Path<String>,
    Json(options): Json<GenModelRelationshipData>,
) -> Result<Json<Option<GenModelRelationship>>, ServerError> {
    let _ = CONN
        .query(&format!(
            "DELETE {0} WHERE provider = '{1}' and model = '{2}';",
            GEN_MODELS_TABLE, options.provider, options.model
        ))
        .await?;
    Ok(Json(
        CONN.update((GEN_MODELS_TABLE, id.trim()))
            .content(options)
            .await?,
    ))
}

pub async fn get_gen_models(
    id: Path<String>,
) -> Result<Json<Option<GenModelRelationship>>, ServerError> {
    Ok(Json(CONN.select((GEN_MODELS_TABLE, id.trim())).await?))
}

pub async fn get_default_gen_options_from_model(
    Path((id, model)): Path<(String, String)>,
) -> Result<Json<Option<GenOptions>>, ServerError> {
    Ok(Json(
        CONN.query(&format!(
            "SELECT * FROM {0} WHERE provider = '{1}' and model = '{2}';",
            GEN_MODELS_TABLE,
            id.trim(),
            model.trim()
        ))
        .await?
        .take(0)?,
    ))
}

pub async fn get_models_from_options(
    id: Path<String>,
) -> Result<Json<Vec<SettingsProvider>>, ServerError> {
    Ok(Json(
        CONN.query(&format!(
            "SELECT * FROM {0} WHERE option = '{1}';",
            GEN_MODELS_TABLE,
            id.trim()
        ))
        .await?
        .take(0)?,
    ))
}

pub async fn get_gen_models_from_options(
    id: Path<String>,
) -> Result<Json<Vec<GenModelRelationship>>, ServerError> {
    Ok(Json(
        CONN.query(&format!(
            "SELECT * FROM {0} WHERE option = '{1}';",
            GEN_MODELS_TABLE,
            id.trim()
        ))
        .await?
        .take(0)?,
    ))
}

pub async fn delete_gen_models(
    id: Path<String>,
) -> Result<Json<Option<GenModelRelationship>>, ServerError> {
    Ok(Json(CONN.delete((GEN_MODELS_TABLE, id.trim())).await?))
}

pub async fn list_all_gen_models() -> Result<Json<Vec<GenOptions>>, ServerError> {
    Ok(Json(CONN.select(GEN_MODELS_TABLE).await?))
}
