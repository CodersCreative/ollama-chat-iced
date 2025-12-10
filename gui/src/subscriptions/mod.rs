use std::collections::HashMap;

use iced::{Subscription, Task};
use ochat_types::{providers::ollama::PullModelStreamResult, settings::SettingsProvider};

use crate::{Application, DATA, Message, data::Data, subscriptions::pull::Pull};

pub mod message;
pub mod pull;

#[derive(Debug, Clone)]
pub enum SubMessage {
    Pull(SettingsProvider),
    Pulling(u32, PullModelStreamResult),
    StopPulling(u32),
}

impl SubMessage {
    pub fn handle(self, app: &mut Application) -> Task<Message> {
        match self {
            Self::Pull(model) => {
                let id = app.subscriptions.counter.clone();
                app.subscriptions.counter += 1;
                app.subscriptions
                    .pulls
                    .insert(id, Pull::new(model.provider, model.model));

                app.subscriptions
                    .pulls
                    .get_mut(&id)
                    .unwrap()
                    .start()
                    .map(move |x| {
                        let x = match x {
                            pull::PullUpdate::Pulling(x) => x,
                            pull::PullUpdate::Finished(Ok(_)) => PullModelStreamResult::Finished,
                            pull::PullUpdate::Finished(Err(e)) => PullModelStreamResult::Err(e),
                        };
                        Message::Subscription(SubMessage::Pulling(id, x))
                    })
            }
            Self::Pulling(id, PullModelStreamResult::Finished) => {
                let _ = app.subscriptions.pulls.remove(&id);
                let (url, providers) = {
                    let data = DATA.read().unwrap();
                    (
                        data.instance_url.clone().unwrap(),
                        data.providers
                            .iter()
                            .map(|x| x.id.key().to_string())
                            .collect(),
                    )
                };

                Task::future(async {
                    if let Ok(x) = Data::get_models(url, providers).await {
                        DATA.write().unwrap().models = x;
                    }

                    Message::None
                })
            }
            Self::Pulling(id, result) => {
                if let Some(x) = app.subscriptions.pulls.get_mut(&id) {
                    x.progress(result);
                }

                Task::none()
            }
            Self::StopPulling(id) => {
                app.subscriptions.pulls.remove(&id);
                Task::none()
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Subscriptions {
    pub counter: u32,
    pub pulls: HashMap<u32, Pull>,
}

impl Subscriptions {
    pub fn get(&self, _app: &Application) -> Subscription<Message> {
        Subscription::none()
    }
}
