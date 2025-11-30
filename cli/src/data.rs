use reqwest::Client;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::LazyLock;

static REQWEST_CLIENT: LazyLock<Client> = LazyLock::new(|| Client::new());

pub struct Request(pub String);

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
