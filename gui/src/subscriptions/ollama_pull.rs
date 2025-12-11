use iced::{
    Task,
    futures::StreamExt,
    task::{Straw, sipper},
};
use ochat_types::providers::ollama::{OllamaPullModelResponse, OllamaPullModelStreamResult};

use crate::{DATA, data::REQWEST_CLIENT};

#[derive(Debug, Clone)]
pub struct OllamaPull {
    pub provider: String,
    pub model: String,
    pub state: OllamaPullModelStreamResult,
}

pub enum OllamaPullUpdate {
    Pulling(OllamaPullModelStreamResult),
    Finished(Result<(), String>),
}

impl OllamaPull {
    pub fn new(provider: String, model: String) -> Self {
        Self {
            provider,
            model,
            state: OllamaPullModelStreamResult::Idle,
        }
    }

    pub fn start(&mut self) -> Task<OllamaPullUpdate> {
        match self.state {
            OllamaPullModelStreamResult::Err(_)
            | OllamaPullModelStreamResult::Finished
            | OllamaPullModelStreamResult::Idle => {
                let (task, _handle) = Task::sip(
                    pull_stream(self.provider.clone(), self.model.clone()),
                    OllamaPullUpdate::Pulling,
                    OllamaPullUpdate::Finished,
                )
                .abortable();

                self.state =
                    OllamaPullModelStreamResult::Pulling(OllamaPullModelResponse::default());

                task
            }
            _ => Task::none(),
        }
    }

    pub fn progress(&mut self, progress: OllamaPullModelStreamResult) {
        self.state = progress;
    }
}

pub fn pull_stream(
    provider: String,
    model: String,
) -> impl Straw<(), OllamaPullModelStreamResult, String> {
    let url = DATA.read().unwrap().instance_url.clone().unwrap();

    sipper(async move |mut output| {
        let mut response = REQWEST_CLIENT
            .post(&format!("{0}/provider/{1}/model/{2}", url, provider, model))
            .send()
            .await
            .unwrap()
            .bytes_stream();

        while let Some(status) = response.next().await {
            let _ = match serde_json::from_slice::<OllamaPullModelStreamResult>(&status.unwrap()) {
                Ok(x) => {
                    let _ = output.send(x).await;
                }
                Err(e) => {
                    let _ = output
                        .send(OllamaPullModelStreamResult::Err(e.to_string()))
                        .await;
                }
            };
        }

        let _ = output.send(OllamaPullModelStreamResult::Finished).await;

        Ok(())
    })
}
