use iced::{
    Subscription,
    futures::{SinkExt, Stream, StreamExt, channel::mpsc},
    stream::try_channel,
};
use ochat_types::providers::ollama::{PullModelResponse, PullModelStreamResult};

use crate::{Application, DATA, Message, data::REQWEST_CLIENT};

#[derive(Debug, Clone)]
pub struct Pull {
    pub provider: String,
    pub model: String,
    pub state: PullModelStreamResult,
}

impl Pull {
    pub fn new(provider: String, model: String) -> Self {
        Self {
            provider,
            model,
            state: PullModelStreamResult::Pulling(PullModelResponse {
                status: String::new(),
                digest: None,
                total: None,
                completed: None,
            }),
        }
    }

    pub fn progress(&mut self, progress: PullModelStreamResult) {
        self.state = progress;
    }

    pub fn subscription(&self, _app: &Application, id: u32) -> Subscription<Message> {
        match self.state {
            PullModelStreamResult::Pulling(_) => {
                pull(id, self.model.clone(), self.provider.clone())
                    .map(|x| Message::Subscription(super::SubMessage::Pulling(x.0, x.1)))
            }
            _ => Subscription::none(),
        }
    }
}

pub fn pull(
    id: u32,
    model: String,
    provider: String,
) -> iced::Subscription<(u32, PullModelStreamResult)> {
    Subscription::run_with((id, model, provider), move |(id, model, provider)| {
        pull_stream(*id, provider.to_string(), model.to_string())
    })
}

pub fn pull_stream(
    id: u32,
    provider: String,
    model: String,
) -> impl Stream<Item = (u32, PullModelStreamResult)> {
    let url = DATA.read().unwrap().instance_url.clone().unwrap();

    try_channel(
        1,
        move |mut output: mpsc::Sender<(u32, PullModelStreamResult)>| async move {
            let mut response = REQWEST_CLIENT
                .post(&format!("{0}/provider/{1}/model/{2}", url, provider, model))
                .send()
                .await
                .unwrap()
                .bytes_stream();

            while let Some(status) = response.next().await {
                let _ = match serde_json::from_slice::<PullModelStreamResult>(&status.unwrap()) {
                    Ok(x) => {
                        let _ = output.send((id, x)).await;
                    }
                    Err(e) => {
                        let _ = output
                            .send((id, PullModelStreamResult::Err(e.to_string())))
                            .await;
                    }
                };
            }

            let _ = output.send((id, PullModelStreamResult::Finished)).await;

            Ok(())
        },
    )
    .map(|x: Result<(u32, PullModelStreamResult), String>| x.unwrap())
}
