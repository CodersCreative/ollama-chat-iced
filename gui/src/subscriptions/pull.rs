use iced::{
    Task,
    futures::StreamExt,
    task::{Straw, sipper},
};
use ochat_types::providers::ollama::{PullModelResponse, PullModelStreamResult};

use crate::{DATA, data::REQWEST_CLIENT};

#[derive(Debug, Clone)]
pub struct Pull {
    pub provider: String,
    pub model: String,
    pub state: PullModelStreamResult,
}

pub enum PullUpdate {
    Pulling(PullModelStreamResult),
    Finished(Result<(), String>),
}

impl Pull {
    pub fn new(provider: String, model: String) -> Self {
        Self {
            provider,
            model,
            state: PullModelStreamResult::Idle,
        }
    }

    pub fn start(&mut self) -> Task<PullUpdate> {
        match self.state {
            PullModelStreamResult::Err(_)
            | PullModelStreamResult::Finished
            | PullModelStreamResult::Idle => {
                let (task, handle) = Task::sip(
                    pull_stream(self.provider.clone(), self.model.clone()),
                    PullUpdate::Pulling,
                    PullUpdate::Finished,
                )
                .abortable();

                self.state = PullModelStreamResult::Pulling(PullModelResponse::default());

                task
            }
            _ => Task::none(),
        }
    }

    pub fn progress(&mut self, progress: PullModelStreamResult) {
        self.state = progress;
    }
}

pub fn pull_stream(
    provider: String,
    model: String,
) -> impl Straw<(), PullModelStreamResult, String> {
    let url = DATA.read().unwrap().instance_url.clone().unwrap();

    sipper(async move |mut output| {
        let mut response = REQWEST_CLIENT
            .post(&format!("{0}/provider/{1}/model/{2}", url, provider, model))
            .send()
            .await
            .unwrap()
            .bytes_stream();

        while let Some(status) = response.next().await {
            let _ = match serde_json::from_slice::<PullModelStreamResult>(&status.unwrap()) {
                Ok(x) => {
                    let _ = output.send(x).await;
                }
                Err(e) => {
                    let _ = output.send(PullModelStreamResult::Err(e.to_string())).await;
                }
            };
        }

        let _ = output.send(PullModelStreamResult::Finished).await;

        Ok(())
    })
}
