use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{DeleteModelResponse, Model},
};
use axum::{Json, extract::Path};

use crate::{
    CONN,
    errors::ServerError,
    providers::{PROVIDER_TABLE, Provider},
};

pub async fn list_all_provider_models(id: Path<String>) -> Result<Json<Vec<Model>>, ServerError> {
    let response = if let Some(provider) = CONN
        .select::<Option<Provider>>((PROVIDER_TABLE, &*id))
        .await?
    {
        let provider = Into::<Client<OpenAIConfig>>::into(&provider);
        provider.models().list().await?
    } else {
        panic!()
    };

    Ok(Json(response.data))
}

pub async fn delete_provider_model(
    Path((id, model)): Path<(String, String)>,
) -> Result<Json<DeleteModelResponse>, ServerError> {
    let response = if let Some(provider) = CONN
        .select::<Option<Provider>>((PROVIDER_TABLE, &*id))
        .await?
    {
        let provider = Into::<Client<OpenAIConfig>>::into(&provider);
        provider.models().delete(&*model).await?
    } else {
        panic!()
    };

    Ok(Json(response))
}

pub async fn get_provider_model(
    Path((id, model)): Path<(String, String)>,
) -> Result<Json<Model>, ServerError> {
    let response = if let Some(provider) = CONN
        .select::<Option<Provider>>((PROVIDER_TABLE, &*id))
        .await?
    {
        let provider = Into::<Client<OpenAIConfig>>::into(&provider);
        provider.models().retrieve(&*model).await?
    } else {
        panic!()
    };

    Ok(Json(response))
}
