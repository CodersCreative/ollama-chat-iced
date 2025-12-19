use ochat_types::{
    providers::Provider,
    settings::{SettingsProvider, SettingsProviderBuilder},
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::error::Error;

use crate::get_client;
pub mod settings;
pub mod start;
pub mod versions;

#[derive(Clone, Debug, Default)]
pub struct Data {
    pub instance_url: Option<String>,
    pub providers: Vec<Provider>,
    pub models: Vec<SettingsProvider>,
}

#[derive(Clone, Debug, Default)]
pub struct Request(String);

impl Request {
    pub async fn make_request<T: DeserializeOwned, Json: Serialize>(
        &self,
        endpoint: &str,
        body: &Json,
        request_type: RequestType,
    ) -> Result<T, String> {
        request_ochat_server(&format!("{}/{}", self.0, endpoint,), body, request_type).await
    }
}

unsafe impl Sync for Data {}
unsafe impl Send for Data {}

impl Data {
    pub async fn get(instance: Option<String>) -> Result<Data, Box<dyn Error>> {
        let instance = match instance {
            Some(x) => x,
            _ => String::from("http://localhost:1212"),
        };

        let providers: Vec<Provider> = request_ochat_server(
            &format!("{}/{}", instance, "provider/all/"),
            &(),
            RequestType::Get,
        )
        .await?;

        let models = Self::get_models(
            instance.clone(),
            providers.iter().map(|x| x.id.key().to_string()).collect(),
        )
        .await?;

        Ok(Data {
            instance_url: Some(instance),
            providers,
            models,
        })
    }

    pub async fn get_models(
        url: String,
        providers: Vec<String>,
    ) -> Result<Vec<SettingsProvider>, Box<dyn Error>> {
        let mut models: Vec<SettingsProvider> = Vec::new();

        for provider in providers.iter() {
            let provider_models: Result<Vec<Value>, String> = request_ochat_server(
                &format!("{}/{}", url, format!("provider/{}/model/all/", &provider)),
                &(),
                RequestType::Get,
            )
            .await;

            if let Ok(provider_models) = provider_models {
                for model in provider_models {
                    models.push(
                        SettingsProviderBuilder::default()
                            .provider(provider.to_string())
                            .model(model["id"].as_str().unwrap().to_string())
                            .build()?,
                    );
                }
            }
        }

        let mut hf_models: Vec<SettingsProvider> = request_ochat_server(
            &format!("{}/provider/hf/text/model/downloaded/", url),
            &(),
            RequestType::Get,
        )
        .await?;

        models.append(&mut hf_models);

        Ok(models)
    }

    pub fn to_request(&self) -> Request {
        Request(
            self.instance_url
                .clone()
                .unwrap_or(String::from("http://localhost:1212")),
        )
    }
}

pub enum RequestType {
    Get,
    Post,
    Put,
    Delete,
}

pub async fn request_ochat_server<T: DeserializeOwned, Json: Serialize>(
    url: &str,
    body: &Json,
    request_type: RequestType,
) -> Result<T, String> {
    let request = match request_type {
        RequestType::Get => get_client().get(url),
        RequestType::Post => get_client().post(url),
        RequestType::Put => get_client().put(url),
        RequestType::Delete => get_client().delete(url),
    };

    serde_json::from_value(
        request
            .json(body)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
