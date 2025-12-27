use crate::backend::{
    CONN,
    errors::ServerError,
    providers::{PROVIDER_TABLE, Provider, provider_into_reqwest},
};
use axum::{Json, extract::Path};
use ochat_types::providers::Model;
use serde_json::Value;

pub async fn list_all_provider_models(id: Path<String>) -> Result<Json<Vec<Model>>, ServerError> {
    let response = if let Some(provider) = CONN
        .select::<Option<Provider>>((PROVIDER_TABLE, id.trim()))
        .await?
    {
        match provider_into_reqwest(&provider)
            .build()?
            .get(&format!("{}/models", provider.url.trim()))
            .send()
            .await
        {
            Ok(x) => {
                let value: Value = x.json().await?;
                serde_json::from_value(value.get("data").unwrap().clone()).unwrap()
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    } else {
        return Ok(Json(Vec::new()));
    };

    Ok(Json(response))
}

pub async fn delete_provider_model(
    Path((id, model)): Path<(String, String)>,
) -> Result<Json<String>, ServerError> {
    let response = if let Some(provider) = CONN
        .select::<Option<Provider>>((PROVIDER_TABLE, id.trim()))
        .await?
    {
        match provider_into_reqwest(&provider)
            .build()?
            .delete(&format!("{}/models/{}", provider.url.trim(), model.trim()))
            .send()
            .await
        {
            Ok(x) => x.json().await?,
            Err(e) => {
                return Err(e.into());
            }
        }
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
        match provider_into_reqwest(&provider)
            .build()?
            .get(&format!("{}/models/{}", provider.url.trim(), model.trim()))
            .send()
            .await
        {
            Ok(x) => x.json().await?,
            Err(e) => {
                return Err(e.into());
            }
        }
    } else {
        panic!()
    };

    Ok(Json(response))
}
