use crate::{
    Application, CacheMessage, DATA, Message, PopUp,
    pages::home::sidebar::PreviewMk,
    subscriptions::{
        hf_pull::{HFPull, HFPullUpdate},
        message::{MessageGen, MessageGenUpdate},
        ollama_pull::{OllamaPull, OllamaPullUpdate},
    },
};
use iced::{Subscription, Task, widget::markdown, window};
use ochat_common::data::{Data, RequestType};
use ochat_types::{
    chats::{messages::MessageData, previews::Preview},
    generation::text::{ChatQueryData, ChatStreamResult},
    providers::{
        hf::{HFModel, HFPullModelStreamResult},
        ollama::{OllamaModelsInfo, OllamaPullModelStreamResult},
    },
    settings::SettingsProvider,
};
use std::collections::HashMap;

pub mod hf_pull;
pub mod message;
pub mod ollama_pull;

#[derive(Debug, Clone)]
pub enum SubMessage {
    GenMessage(String, ChatQueryData),
    GeneratingMessage(u32, ChatStreamResult),
    StopGenMessage(u32),
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
            Self::GenMessage(x, query) => {
                let id = app.subscriptions.counter.clone();
                app.subscriptions.counter += 1;
                app.subscriptions
                    .message_gens
                    .insert(id, MessageGen::new(x, query));

                app.subscriptions
                    .message_gens
                    .get_mut(&id)
                    .unwrap()
                    .start()
                    .map(move |x| {
                        let x = match x {
                            MessageGenUpdate::Generating(x) => x,
                            MessageGenUpdate::Finished(Ok(_)) => ChatStreamResult::Finished,
                            MessageGenUpdate::Finished(Err(e)) => ChatStreamResult::Err(e),
                        };
                        Message::Subscription(SubMessage::GeneratingMessage(id, x))
                    })
            }
            Self::GeneratingMessage(id, ChatStreamResult::Finished) => {
                let message_id = if let Some(x) = app.subscriptions.message_gens.remove(&id) {
                    x.id
                } else {
                    return Task::none();
                };

                let chat_id = app
                    .view_data
                    .home
                    .chats
                    .iter()
                    .find(|x| x.1.messages.contains(&message_id))
                    .map(|x| x.1.chat.id.key().to_string());

                Task::future(async {
                    let req = DATA.read().unwrap().to_request();

                    if let Some(id) = chat_id {
                        match req
                            .make_request::<Preview, ()>(
                                &format!("preview/{}", id),
                                &(),
                                RequestType::Put,
                            )
                            .await
                        {
                            Ok(x) => Message::Cache(CacheMessage::AddPreview(PreviewMk::from(x))),
                            Err(e) => Message::Err(e),
                        }
                    } else {
                        Message::None
                    }
                })
            }
            Self::GeneratingMessage(id, ChatStreamResult::Generated(result)) => {
                let key = if let Some(x) = app.subscriptions.message_gens.get_mut(&id) {
                    x.progress(ChatStreamResult::Generated(result.clone()));
                    x.id.clone()
                } else {
                    return Task::none();
                };

                let (id, msg) = if let Some(msg) = app.cache.home_shared.messages.0.get_mut(&key) {
                    msg.content = markdown::Content::parse(&result.content);
                    msg.thinking = result
                        .thinking
                        .as_ref()
                        .map(|x| markdown::Content::parse(&x));
                    msg.base.content = result.content;
                    msg.base.thinking = result.thinking;
                    (
                        msg.base.id.key().to_string(),
                        Into::<MessageData>::into(msg.base.clone()),
                    )
                } else {
                    return Task::none();
                };

                Task::future(async move {
                    let req = DATA.read().unwrap().to_request();

                    match req
                        .make_request::<ochat_types::chats::messages::Message, MessageData>(
                            &format!("message/{}", id),
                            &msg,
                            RequestType::Put,
                        )
                        .await
                    {
                        Ok(_) => Message::None,
                        Err(e) => Message::Err(e),
                    }
                })
            }
            Self::GeneratingMessage(id, ChatStreamResult::Generating(result)) => {
                let key = if let Some(x) = app.subscriptions.message_gens.get_mut(&id) {
                    x.progress(ChatStreamResult::Generating(result.clone()));
                    x.id.clone()
                } else {
                    return Task::none();
                };

                if let Some(msg) = app.cache.home_shared.messages.0.get_mut(&key) {
                    msg.base.content.push_str(&result.content);

                    if let (Some(thinking), Some(add)) = (&mut msg.base.thinking, &result.thinking)
                    {
                        thinking.push_str(&add);
                    } else {
                        msg.base.thinking = result.thinking;
                    }

                    msg.content = markdown::Content::parse(&msg.base.content);
                    msg.thinking = msg
                        .base
                        .thinking
                        .as_ref()
                        .map(|x| markdown::Content::parse(&x));
                }

                Task::none()
            }
            Self::GeneratingMessage(id, result) => {
                if let ChatStreamResult::Err(e) = &result {
                    app.add_popup(PopUp::Err(e.to_string()));
                }
                if let Some(x) = app.subscriptions.message_gens.get_mut(&id) {
                    x.progress(result);
                }

                Task::none()
            }
            Self::StopGenMessage(id) => {
                app.subscriptions.message_gens.remove(&id);
                Task::none()
            }
            Self::OllamaPull(data, model) => {
                let id = app.subscriptions.counter.clone();
                app.cache.home_shared.downloads.ollama.insert(id, data);
                app.subscriptions.counter += 1;
                app.subscriptions
                    .ollama_pulls
                    .insert(id, OllamaPull::new(model.provider, model.model));

                app.add_pull_pop_up(id);

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
                    let jwt = DATA.read().unwrap().jwt.clone();
                    match Data::get_models(&jwt, url, providers).await {
                        Ok(x) => {
                            DATA.write().unwrap().models = x;
                            Message::None
                        }
                        Err(e) => Message::Err(e.to_string()),
                    }
                })
            }
            Self::OllamaPulling(id, result) => {
                if let OllamaPullModelStreamResult::Err(e) = &result {
                    app.add_popup(PopUp::Err(e.to_string()));
                }
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

                app.add_pull_pop_up(id);

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
                    let jwt = DATA.read().unwrap().jwt.clone();
                    match Data::get_models(&jwt, url, providers).await {
                        Ok(x) => {
                            DATA.write().unwrap().models = x;
                            Message::None
                        }
                        Err(e) => Message::Err(e.to_string()),
                    }
                })
            }
            Self::HFPulling(id, result) => {
                if let HFPullModelStreamResult::Err(e) = &result {
                    app.add_popup(PopUp::Err(e.to_string()));
                }
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
    pub message_gens: HashMap<u32, MessageGen>,
}

impl Subscriptions {
    pub fn get(&self, _app: &Application) -> Subscription<Message> {
        window::resize_events()
            .map(|x| Message::Window(crate::windows::message::WindowMessage::Resize(x.0, x.1)))
    }
}
