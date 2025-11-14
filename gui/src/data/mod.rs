use std::{error::Error, sync::LazyLock};

use ochat_types::{
    providers::Provider,
    settings::{SettingsProvider, SettingsProviderBuilder},
};
use reqwest::Client;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

static REQWEST_CLIENT: LazyLock<Client> = LazyLock::new(|| Client::new());

#[derive(Clone, Debug, Default)]
pub struct Data {
    pub instance_url: Option<String>,
    pub providers: Vec<Provider>,
    pub models: Vec<SettingsProvider>,
}

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

        let mut models: Vec<SettingsProvider> = Vec::new();

        for provider in providers.iter() {
            let provider_models: Result<Vec<Value>, String> = request_ochat_server(
                &format!(
                    "{}/{}",
                    instance,
                    format!("provider/{}/model/all/", provider.id.key())
                ),
                &(),
                RequestType::Get,
            )
            .await;

            if let Ok(provider_models) = provider_models {
                for model in provider_models {
                    models.push(
                        SettingsProviderBuilder::default()
                            .provider(provider.id.key().to_string())
                            .model(model["id"].as_str().unwrap().to_string())
                            .build()?,
                    );
                }
            } else {
                eprintln!("{:?}", provider_models);
            }
        }

        Ok(Data {
            instance_url: Some(instance),
            providers,
            models,
        })
    }

    pub fn to_request(&self) -> Request {
        Request(self.instance_url.clone().unwrap())
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
        RequestType::Get => REQWEST_CLIENT.get(url),
        RequestType::Post => REQWEST_CLIENT.post(url),
        RequestType::Put => REQWEST_CLIENT.put(url),
        RequestType::Delete => REQWEST_CLIENT.delete(url),
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
