use ochat_types::{
    providers::Provider,
    settings::{SettingsProvider, SettingsProviderBuilder},
};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::error::Error;

pub mod start;
pub mod versions;

#[derive(Clone, Debug, Default)]
pub struct Data {
    pub instance_url: Option<String>,
    pub providers: Vec<Provider>,
    pub models: Vec<SettingsProvider>,
    pub jwt: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Request {
    pub url: String,
    pub jwt: Option<String>,
}

impl Request {
    pub async fn make_request<T: DeserializeOwned, Json: Serialize>(
        &self,
        endpoint: &str,
        body: &Json,
        request_type: RequestType,
    ) -> Result<T, String> {
        request_ochat_server(
            &self.jwt,
            &format!("{}/{}", self.url, endpoint,),
            body,
            request_type,
        )
        .await
    }

    pub fn get_client(&self) -> reqwest::Client {
        get_client(&self.jwt)
    }
}

unsafe impl Sync for Data {}
unsafe impl Send for Data {}

impl Data {
    pub async fn get(
        instance: Option<String>,
        jwt: Option<String>,
    ) -> Result<Data, Box<dyn Error>> {
        let instance = match instance {
            Some(x) => x,
            _ => String::from("http://localhost:1212/api"),
        };

        let providers: Vec<Provider> = if jwt.is_some() {
            request_ochat_server(
                &jwt,
                &format!("{}/{}", instance, "provider/all/"),
                &(),
                RequestType::Get,
            )
            .await?
        } else {
            Vec::new()
        };

        let models = Self::get_models(
            &jwt,
            instance.clone(),
            providers.iter().map(|x| x.id.key().to_string()).collect(),
        )
        .await?;

        Ok(Data {
            instance_url: Some(instance),
            providers,
            models,
            jwt,
        })
    }

    pub async fn get_models(
        jwt: &Option<String>,
        url: String,
        providers: Vec<String>,
    ) -> Result<Vec<SettingsProvider>, Box<dyn Error>> {
        if jwt.is_none() {
            return Ok(Vec::new());
        }
        let mut models: Vec<SettingsProvider> = Vec::new();

        for provider in providers.iter() {
            let provider_models: Result<Vec<Value>, String> = request_ochat_server(
                jwt,
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
            jwt,
            &format!("{}/provider/hf/text/model/downloaded/", url),
            &(),
            RequestType::Get,
        )
        .await?;

        models.append(&mut hf_models);

        Ok(models)
    }

    pub fn to_request(&self) -> Request {
        Request {
            url: self
                .instance_url
                .clone()
                .unwrap_or(String::from("http://localhost:1212")),
            jwt: self.jwt.clone(),
        }
    }

    pub fn get_client(&self) -> reqwest::Client {
        get_client(&self.jwt)
    }
}

pub enum RequestType {
    Get,
    Post,
    Put,
    Delete,
}

pub fn get_client(jwt: &Option<String>) -> reqwest::Client {
    if let Some(jwt) = jwt {
        let mut headers = HeaderMap::new();
        headers.append("Authorization", HeaderValue::from_str(jwt).unwrap());
        reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap()
    } else {
        reqwest::Client::new()
    }
}

pub async fn request_ochat_server<T: DeserializeOwned, Json: Serialize>(
    jwt: &Option<String>,
    url: &str,
    body: &Json,
    request_type: RequestType,
) -> Result<T, String> {
    let request = get_client(jwt);

    let request = match request_type {
        RequestType::Get => request.get(url),
        RequestType::Post => request.post(url),
        RequestType::Put => request.put(url),
        RequestType::Delete => request.delete(url),
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
