use std::{error::Error, sync::LazyLock};

use ochat_types::{
    providers::Provider,
    settings::{SettingsProvider, SettingsProviderBuilder},
};
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;

static REQWEST_CLIENT: LazyLock<Client> = LazyLock::new(|| Client::new());

#[derive(Clone, Debug, Default)]
pub struct Data {
    pub instance_url: Option<String>,
    pub providers: Vec<Provider>,
    pub models: Vec<SettingsProvider>,
}

impl Data {
    pub async fn get(instance: Option<String>) -> Result<Data, Box<dyn Error>> {
        let instance = match instance {
            Some(x) => x,
            _ => String::from("http://localhost:1212"),
        };

        let providers: Vec<Provider> = serde_json::from_value(
            request_ochat_server(&format!("{}/{}", instance, "provider/all/"), &()).await?,
        )?;

        let mut models: Vec<SettingsProvider> = Vec::new();

        for provider in providers.iter() {
            let provider_models: Vec<Value> = serde_json::from_value(
                request_ochat_server(
                    &format!(
                        "{}/{}",
                        instance,
                        format!("provider/{}/model/all/", provider.id.key())
                    ),
                    &(),
                )
                .await?,
            )?;

            for model in provider_models {
                models.push(
                    SettingsProviderBuilder::default()
                        .provider(provider.id.key().to_string())
                        .model(serde_json::from_value(model["id"].clone())?)
                        .build()?,
                );
            }
        }

        Ok(Data {
            instance_url: Some(instance),
            providers,
            models,
        })
    }

    pub async fn make_request<T: Serialize>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<Value, Box<dyn Error>> {
        request_ochat_server(
            &format!("{}/{}", self.instance_url.as_ref().unwrap(), endpoint),
            body,
        )
        .await
    }
}

pub async fn request_ochat_server<T: Serialize>(
    url: &str,
    body: &T,
) -> Result<Value, Box<dyn Error>> {
    Ok(REQWEST_CLIENT
        .get(url)
        .json(body)
        .send()
        .await?
        .json()
        .await?)
}
