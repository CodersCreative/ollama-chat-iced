use axum::{Json, extract::Path};
use ochat_types::{
    options::{
        GenOptions,
        relationships::{GenModelRelationship, GenModelRelationshipData},
    },
    settings::SettingsProvider,
};

use crate::{CONN, errors::ServerError, options::GEN_OPTIONS_TABLE};

pub const GEN_MODELS_TABLE: &str = "gen_models";

pub async fn define_gen_models() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS;
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
    let relationship: Option<GenModelRelationship> =
        CONN.create(GEN_OPTIONS_TABLE).content(relationship).await?;

    Ok(Json(relationship))
}

pub async fn update_gen_models(
    id: Path<String>,
    Json(options): Json<GenModelRelationshipData>,
) -> Result<Json<Option<GenModelRelationship>>, ServerError> {
    let _ = CONN.query(&format!(
        "DELETE {0} WHERE provider = '{1}' and model = '{2}';",
        GEN_MODELS_TABLE, options.provider, options.model
    ));

    let relationship = CONN
        .update((GEN_MODELS_TABLE, &*id))
        .content(options)
        .await?;

    Ok(Json(relationship))
}

pub async fn get_gen_models(
    id: Path<String>,
) -> Result<Json<Option<GenModelRelationship>>, ServerError> {
    let relationship = CONN.select((GEN_MODELS_TABLE, &*id)).await?;
    Ok(Json(relationship))
}

pub async fn get_default_gen_options_from_model(
    Path((id, model)): Path<(String, String)>,
) -> Result<Json<Option<GenOptions>>, ServerError> {
    let options = CONN
        .query(&format!(
            "SELECT * FROM {0} WHERE provider = '{1}' and model = '{2}';",
            GEN_MODELS_TABLE, &*id, &*model
        ))
        .await?
        .take(0)?;

    Ok(Json(options))
}

pub async fn get_models_from_options(
    id: Path<String>,
) -> Result<Json<Vec<SettingsProvider>>, ServerError> {
    let models = CONN
        .query(&format!(
            "SELECT * FROM {0} WHERE option = '{1}';",
            GEN_MODELS_TABLE, &*id
        ))
        .await?
        .take(0)?;

    Ok(Json(models))
}

pub async fn get_gen_models_from_options(
    id: Path<String>,
) -> Result<Json<Vec<GenModelRelationship>>, ServerError> {
    let models = CONN
        .query(&format!(
            "SELECT * FROM {0} WHERE option = '{1}';",
            GEN_MODELS_TABLE, &*id
        ))
        .await?
        .take(0)?;

    Ok(Json(models))
}

pub async fn delete_gen_models(
    id: Path<String>,
) -> Result<Json<Option<GenModelRelationship>>, ServerError> {
    let relationship = CONN.delete((GEN_OPTIONS_TABLE, &*id)).await?;
    Ok(Json(relationship))
}

pub async fn list_all_gen_models() -> Result<Json<Vec<GenOptions>>, ServerError> {
    let relationships = CONN.select(GEN_OPTIONS_TABLE).await?;
    Ok(Json(relationships))
}
