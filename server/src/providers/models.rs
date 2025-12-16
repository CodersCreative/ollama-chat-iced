use async_openai::types::DeleteModelResponse;
use axum::{Json, extract::Path};
use ochat_types::providers::Model;
use serde_json::Value;

use crate::{
    CONN,
    errors::ServerError,
    providers::{PROVIDER_TABLE, Provider, provider_into_config},
};

pub async fn list_all_provider_models(id: Path<String>) -> Result<Json<Vec<Model>>, ServerError> {
    let response = if let Some(provider) = CONN
        .select::<Option<Provider>>((PROVIDER_TABLE, id.trim()))
        .await?
    {
        match {
            let provider = provider_into_config(&provider);
            provider.models().list().await
        } {
            Ok(x) => x
                .data
                .into_iter()
                .map(|x| Model {
                    id: x.id,
                    object: Some(x.object),
                    created: Some(x.created),
                    owned_by: Some(x.owned_by),
                })
                .collect(),
            Err(e) => {
                if let Ok(x) = reqwest::Client::new()
                    .get(&format!("{}/models", provider.url.trim()))
                    .send()
                    .await
                {
                    let value: Value = x.json().await?;
                    serde_json::from_value(value.get("data").unwrap().clone()).unwrap()
                } else {
                    return Err(e.into());
                }
            }
        }
    } else {
        return Ok(Json(Vec::new()));
    };

    Ok(Json(response))
}

pub async fn delete_provider_model(
    Path((id, model)): Path<(String, String)>,
) -> Result<Json<DeleteModelResponse>, ServerError> {
    let response = if let Some(provider) = CONN
        .select::<Option<Provider>>((PROVIDER_TABLE, id.trim()))
        .await?
    {
        let provider = provider_into_config(&provider);
        provider.models().delete(model.trim()).await?
    } else {
        panic!()
    };

    Ok(Json(response))
}

pub async fn get_provider_model(
    Path((id, model)): Path<(String, String)>,
) -> Result<Json<Model>, ServerError> {
    let response = if let Some(provider) = CONN
        .select::<Option<Provider>>((PROVIDER_TABLE, id.trim()))
        .await?
    {
        match {
            let provider = provider_into_config(&provider);
            provider.models().retrieve(model.trim()).await
        } {
            Ok(x) => Model {
                id: x.id,
                object: Some(x.object),
                created: Some(x.created),
                owned_by: Some(x.owned_by),
            },
            Err(e) => {
                if let Ok(x) = reqwest::Client::new()
                    .get(&format!("{}/models/{}", provider.url.trim(), model.trim()))
                    .send()
                    .await
                {
                    x.json().await?
                } else {
                    return Err(e.into());
                }
            }
        }
    } else {
        panic!()
    };

    Ok(Json(response))
}
