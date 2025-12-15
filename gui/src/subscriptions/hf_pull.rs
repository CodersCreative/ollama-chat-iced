use iced::{
    Task,
    futures::StreamExt,
    task::{Straw, sipper},
};
use ochat_types::providers::hf::{HFPullModelResponse, HFPullModelStreamResult};

use crate::{DATA, data::REQWEST_CLIENT};

#[derive(Debug, Clone)]
pub struct HFPull {
    pub name: String,
    pub model: String,
    pub state: HFPullModelStreamResult,
}

pub enum HFPullUpdate {
    Pulling(HFPullModelStreamResult),
    Finished(Result<(), String>),
}

impl HFPull {
    pub fn new(model: String, name: String) -> Self {
        Self {
            name,
            model,
            state: HFPullModelStreamResult::Idle,
        }
    }

    pub fn get_percent(&self) -> f32 {
        match &self.state {
            HFPullModelStreamResult::Pulling(status) => {
                if let (Some(total), Some(completed)) = (&status.total, &status.completed) {
                    (*completed as f64 / *total as f64) as f32 * 100.0
                } else {
                    0.0
                }
            }
            HFPullModelStreamResult::Finished => 100.0,
            _ => 0.0,
        }
    }

    pub fn start(&mut self) -> Task<HFPullUpdate> {
        match self.state {
            HFPullModelStreamResult::Err(_)
            | HFPullModelStreamResult::Finished
            | HFPullModelStreamResult::Idle => {
                let (task, _handle) = Task::sip(
                    pull_stream(self.model.clone(), self.name.clone()),
                    HFPullUpdate::Pulling,
                    HFPullUpdate::Finished,
                )
                .abortable();

                self.state = HFPullModelStreamResult::Pulling(HFPullModelResponse::default());

                task
            }
            _ => Task::none(),
        }
    }

    pub fn progress(&mut self, progress: HFPullModelStreamResult) {
        self.state = progress;
    }
}

pub fn pull_stream(model: String, name: String) -> impl Straw<(), HFPullModelStreamResult, String> {
    let url = DATA.read().unwrap().instance_url.clone().unwrap();

    sipper(async move |mut output| {
        let mut response = REQWEST_CLIENT
            .post(&format!("{0}/provider/hf/model/{1}/{2}", url, model, name))
            .send()
            .await
            .unwrap()
            .bytes_stream();

        while let Some(status) = response.next().await {
            let _ = match serde_json::from_slice::<HFPullModelStreamResult>(&status.unwrap()) {
                Ok(x) => {
                    let _ = output.send(x).await;
                }
                Err(e) => {
                    let _ = output
                        .send(HFPullModelStreamResult::Err(e.to_string()))
                        .await;
                }
            };
        }

        let _ = output.send(HFPullModelStreamResult::Finished).await;

        Ok(())
    })
}
