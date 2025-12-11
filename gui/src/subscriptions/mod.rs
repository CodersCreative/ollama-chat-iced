use std::collections::HashMap;

use iced::{Subscription, Task};
use ochat_types::{
    providers::{
        hf::{HFModel, HFPullModelStreamResult},
        ollama::{OllamaModelsInfo, OllamaPullModelStreamResult},
    },
    settings::SettingsProvider,
};

use crate::{
    Application, DATA, Message,
    data::Data,
    subscriptions::{
        hf_pull::{HFPull, HFPullUpdate},
        ollama_pull::{OllamaPull, OllamaPullUpdate},
    },
};

pub mod hf_pull;
pub mod message;
pub mod ollama_pull;

#[derive(Debug, Clone)]
pub enum SubMessage {
    OllamaPull(OllamaModelsInfo, SettingsProvider),
    OllamaPulling(u32, OllamaPullModelStreamResult),
    OllamaStopPulling(u32),
    HFPull(HFModel, String),
    HFPulling(u32, HFPullModelStreamResult),
    HFStopPulling(u32),
}

impl SubMessage {
    pub fn handle(self, app: &mut Application) -> Task<Message> {
        match self {
            Self::OllamaPull(data, model) => {
                let id = app.subscriptions.counter.clone();
                app.cache.home_shared.downloads.ollama.insert(id, data);
                app.subscriptions.counter += 1;
                app.subscriptions
                    .ollama_pulls
                    .insert(id, OllamaPull::new(model.provider, model.model));

                app.subscriptions
                    .ollama_pulls
                    .get_mut(&id)
                    .unwrap()
                    .start()
                    .map(move |x| {
                        let x = match x {
                            OllamaPullUpdate::Pulling(x) => x,
                            OllamaPullUpdate::Finished(Ok(_)) => {
                                OllamaPullModelStreamResult::Finished
                            }
                            OllamaPullUpdate::Finished(Err(e)) => {
                                OllamaPullModelStreamResult::Err(e)
                            }
                        };
                        Message::Subscription(SubMessage::OllamaPulling(id, x))
                    })
            }
            Self::OllamaPulling(id, OllamaPullModelStreamResult::Finished) => {
                let _ = app.subscriptions.ollama_pulls.remove(&id);
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
            Self::OllamaPulling(id, result) => {
                if let Some(x) = app.subscriptions.ollama_pulls.get_mut(&id) {
                    x.progress(result);
                }

                Task::none()
            }
            Self::OllamaStopPulling(id) => {
                app.subscriptions.ollama_pulls.remove(&id);
                app.cache.home_shared.downloads.ollama.remove(&id);
                Task::none()
            }
            Self::HFPull(model, name) => {
                let id = app.subscriptions.counter.clone();
                app.subscriptions.counter += 1;
                app.subscriptions
                    .hf_pulls
                    .insert(id, HFPull::new(model.id.clone(), name));
                app.cache.home_shared.downloads.hf.insert(id, model);

                app.subscriptions
                    .hf_pulls
                    .get_mut(&id)
                    .unwrap()
                    .start()
                    .map(move |x| {
                        let x = match x {
                            HFPullUpdate::Pulling(x) => x,
                            HFPullUpdate::Finished(Ok(_)) => HFPullModelStreamResult::Finished,
                            HFPullUpdate::Finished(Err(e)) => HFPullModelStreamResult::Err(e),
                        };
                        Message::Subscription(SubMessage::HFPulling(id, x))
                    })
            }
            Self::HFPulling(id, HFPullModelStreamResult::Finished) => {
                let _ = app.subscriptions.ollama_pulls.remove(&id);
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
            Self::HFPulling(id, result) => {
                if let Some(x) = app.subscriptions.hf_pulls.get_mut(&id) {
                    x.progress(result);
                }

                Task::none()
            }
            Self::HFStopPulling(id) => {
                app.subscriptions.hf_pulls.remove(&id);
                app.cache.home_shared.downloads.hf.remove(&id);
                Task::none()
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Subscriptions {
    pub counter: u32,
    pub ollama_pulls: HashMap<u32, OllamaPull>,
    pub hf_pulls: HashMap<u32, HFPull>,
}

impl Subscriptions {
    pub fn get(&self, _app: &Application) -> Subscription<Message> {
        Subscription::none()
    }
}
