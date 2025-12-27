use rig::client::{
    BearerAuth, Capabilities, Capable, DebugExt, Nothing, Provider, ProviderBuilder,
};
use rig::http_client::{self, HttpClientExt};
use rig::providers::ollama::{Message, ToolDefinition};
use rig::providers::openai::{
    CompletionResponse as Response, StreamingCompletionResponse as StreamingResponse,
    send_compatible_streaming_request,
};
use rig::{
    completion::{self, CompletionError, CompletionRequest},
    embeddings::{self, EmbeddingError},
    streaming,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::convert::TryFrom;

type GenericApiKey = BearerAuth;

pub type Client<H = reqwest::Client> = rig::client::Client<GenericExt, H>;
pub type ClientBuilder<H = reqwest::Client> =
    rig::client::ClientBuilder<GenericBuilder, GenericApiKey, H>;

#[derive(Debug, Default, Clone, Copy)]
pub struct GenericExt;

#[derive(Debug, Default, Clone, Copy)]
pub struct GenericBuilder;

impl Provider for GenericExt {
    type Builder = GenericBuilder;

    const VERIFY_PATH: &'static str = "models";

    fn build<H>(
        _: &rig::client::ClientBuilder<
            Self::Builder,
            <Self::Builder as rig::client::ProviderBuilder>::ApiKey,
            H,
        >,
    ) -> http_client::Result<Self> {
        Ok(Self)
    }
}

impl<H> Capabilities<H> for GenericExt {
    type Completion = Capable<CompletionModel<H>>;
    type Transcription = Nothing;
    type Embeddings = Capable<EmbeddingModel<H>>;
}

impl DebugExt for GenericExt {}

impl ProviderBuilder for GenericBuilder {
    type Output = GenericExt;
    type ApiKey = GenericApiKey;

    const BASE_URL: &'static str = "";
}

// ---------- API Error and Response Structures ----------

#[derive(Debug, Deserialize)]
struct ApiErrorResponse {
    message: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ApiResponse<T> {
    Ok(T),
    Err(ApiErrorResponse),
}

// ---------- Embedding API ----------

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub model: String,
    pub embeddings: Vec<Vec<f64>>,
    #[serde(default)]
    pub total_duration: Option<u64>,
    #[serde(default)]
    pub load_duration: Option<u64>,
    #[serde(default)]
    pub prompt_eval_count: Option<u64>,
}

impl From<ApiErrorResponse> for EmbeddingError {
    fn from(err: ApiErrorResponse) -> Self {
        EmbeddingError::ProviderError(err.message)
    }
}

impl From<ApiResponse<EmbeddingResponse>> for Result<EmbeddingResponse, EmbeddingError> {
    fn from(value: ApiResponse<EmbeddingResponse>) -> Self {
        match value {
            ApiResponse::Ok(response) => Ok(response),
            ApiResponse::Err(err) => Err(EmbeddingError::ProviderError(err.message)),
        }
    }
}

// ---------- Embedding Model ----------

#[derive(Clone)]
pub struct EmbeddingModel<T> {
    client: Client<T>,
    pub model: String,
    ndims: usize,
}

impl<T> EmbeddingModel<T> {
    pub fn new(client: Client<T>, model: impl Into<String>, ndims: usize) -> Self {
        Self {
            client,
            model: model.into(),
            ndims,
        }
    }

    pub fn with_model(client: Client<T>, model: &str, ndims: usize) -> Self {
        Self {
            client,
            model: model.into(),
            ndims,
        }
    }
}

impl<T> embeddings::EmbeddingModel for EmbeddingModel<T>
where
    T: HttpClientExt + Clone + 'static,
{
    type Client = Client<T>;

    fn make(client: &Self::Client, model: impl Into<String>, dims: Option<usize>) -> Self {
        Self::new(client.clone(), model, dims.unwrap())
    }

    const MAX_DOCUMENTS: usize = 1024;
    fn ndims(&self) -> usize {
        self.ndims
    }

    async fn embed_texts(
        &self,
        documents: impl IntoIterator<Item = String>,
    ) -> Result<Vec<embeddings::Embedding>, EmbeddingError> {
        let docs: Vec<String> = documents.into_iter().collect();

        let body = serde_json::to_vec(&json!({
            "model": self.model,
            "input": docs
        }))?;

        let req = self
            .client
            .post("embeddings")?
            .body(body)
            .map_err(|e| EmbeddingError::HttpError(e.into()))?;

        let response = self.client.send(req).await?;

        if !response.status().is_success() {
            let text = http_client::text(response).await?;
            return Err(EmbeddingError::ProviderError(text));
        }

        let bytes: Vec<u8> = response.into_body().await?;

        let api_resp: EmbeddingResponse = serde_json::from_slice(&bytes)?;

        if api_resp.embeddings.len() != docs.len() {
            return Err(EmbeddingError::ResponseError(
                "Number of returned embeddings does not match input".into(),
            ));
        }
        Ok(api_resp
            .embeddings
            .into_iter()
            .zip(docs.into_iter())
            .map(|(vec, document)| embeddings::Embedding { document, vec })
            .collect())
    }
}

// ---------- Completion API ----------

#[derive(Clone)]
pub struct CompletionModel<T = reqwest::Client> {
    client: Client<T>,
    pub model: String,
}

impl<T> CompletionModel<T> {
    pub fn new(client: Client<T>, model: &str) -> Self {
        Self {
            client,
            model: model.to_owned(),
        }
    }
}

// ---------- CompletionModel Implementation ----------

#[derive(Debug, Serialize, Deserialize)]
pub struct GenericCompletionRequest {
    model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<ToolDefinition>,
    pub stream: bool,
    think: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u64>,
    options: serde_json::Value,
}
pub fn merge_json(a: serde_json::Value, b: serde_json::Value) -> serde_json::Value {
    match (a, b) {
        (serde_json::Value::Object(mut a_map), serde_json::Value::Object(b_map)) => {
            b_map.into_iter().for_each(|(key, value)| {
                a_map.insert(key, value);
            });
            serde_json::Value::Object(a_map)
        }
        (a, _) => a,
    }
}

impl TryFrom<(&str, CompletionRequest)> for GenericCompletionRequest {
    type Error = CompletionError;

    fn try_from((model, req): (&str, CompletionRequest)) -> Result<Self, Self::Error> {
        // Build up the order of messages (context, chat_history, prompt)
        let mut partial_history = vec![];
        if let Some(docs) = req.normalized_documents() {
            partial_history.push(docs);
        }
        partial_history.extend(req.chat_history);

        // Add preamble to chat history (if available)
        let mut full_history: Vec<Message> = match &req.preamble {
            Some(preamble) => vec![Message::system(preamble)],
            None => vec![],
        };

        // Convert and extend the rest of the history
        full_history.extend(
            partial_history
                .into_iter()
                .map(rig::message::Message::try_into)
                .collect::<Result<Vec<Vec<Message>>, _>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
        );

        let mut think = false;

        // TODO: Fix this up to include the full range of ollama options
        let options = if let Some(mut extra) = req.additional_params {
            if extra.get("think").is_some() {
                think = extra["think"].take().as_bool().ok_or_else(|| {
                    CompletionError::RequestError("`think` must be a bool".into())
                })?;
            }
            merge_json(json!({ "temperature": req.temperature }), extra)
        } else {
            json!({ "temperature": req.temperature })
        };

        Ok(Self {
            model: model.to_string(),
            messages: full_history,
            temperature: req.temperature,
            max_tokens: req.max_tokens,
            stream: false,
            think,
            tools: req
                .tools
                .clone()
                .into_iter()
                .map(ToolDefinition::from)
                .collect::<Vec<_>>(),
            options,
        })
    }
}

impl<T> completion::CompletionModel for CompletionModel<T>
where
    T: HttpClientExt + Clone + Default + std::fmt::Debug + Send + 'static,
{
    type Response = Response;
    type StreamingResponse = StreamingResponse;

    type Client = Client<T>;

    fn make(client: &Self::Client, model: impl Into<String>) -> Self {
        Self::new(client.clone(), model.into().as_str())
    }

    async fn completion(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<completion::CompletionResponse<Self::Response>, CompletionError> {
        let request =
            GenericCompletionRequest::try_from((self.model.as_ref(), completion_request))?;

        let body = serde_json::to_vec(&request)?;

        let req = self
            .client
            .post("chat/completions")?
            .body(body)
            .map_err(http_client::Error::from)?;

        let async_block = async move {
            let response = self.client.send::<_, bytes::Bytes>(req).await?;

            let status = response.status();
            let response_body = response.into_body().into_future().await?.to_vec();

            if status.is_success() {
                match serde_json::from_slice::<ApiResponse<Response>>(&response_body)? {
                    ApiResponse::Ok(response) => response.try_into(),
                    ApiResponse::Err(err) => Err(CompletionError::ProviderError(err.message)),
                }
            } else {
                Err(CompletionError::ProviderError(
                    String::from_utf8_lossy(&response_body).to_string(),
                ))
            }
        };

        async_block.await
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<streaming::StreamingCompletionResponse<Self::StreamingResponse>, CompletionError>
    {
        let mut request = GenericCompletionRequest::try_from((self.model.as_ref(), request))?;
        request.stream = true;

        let body = serde_json::to_vec(&request)?;

        let req = self
            .client
            .post("chat/completions")?
            .body(body)
            .map_err(http_client::Error::from)?;

        send_compatible_streaming_request(self.client.clone(), req).await
    }
}
