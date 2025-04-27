use super::chat::{Chat, ChatBuilder};
use crate::chats::{Chats, State};
use crate::common::Id;
use crate::llm::Tools;
use crate::sound::{get_audio, transcribe};
use crate::utils::get_preview;
use crate::{ChatApp, Message, SAVE_FILE};
use iced::widget::{markdown, text_editor};
use iced::Task;
use kalosm_sound::{rodio::buffer::SamplesBuffer, MicInput};
use ollama_rs::generation::chat::ChatMessage;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use std::{path::PathBuf, sync::Arc};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SavedChats(pub Vec<Chat>, pub Vec<Tools>, pub SystemTime);

#[derive(Default, Debug)]
pub struct TooledOptions {
    pub chats: Vec<ChatMessage>,
    pub tools: Vec<Tools>,
}

#[derive(Debug, Clone)]
pub enum ChatsMessage {
    Regenerate,
    Submit,
    ChangeModel(String),
    Action(text_editor::Action),
    ChangeStart(String),
    ChangeChat(Id),
    NewChat,
    Listen,
    Convert(Option<SamplesBuffer<f32>>),
    Listened(Result<String, String>),
    PickedImage(Result<Vec<PathBuf>, String>),
    PickImage,
    RemoveImage(PathBuf),
}

impl ChatsMessage {
    pub fn handle(&self, id: Id, app: &mut ChatApp) -> Task<Message> {
        match self {
            Self::Regenerate => {
                let saved_id = app.main_view.chats().get(&id).unwrap().saved_chat().clone();

                for x in app.save.chats.iter_mut().filter(|x| x.0 == &saved_id) {
                    x.1 .0.remove(x.1 .0.len() - 1);
                    break;
                }

                app.main_view.update_chat_by_saved(&saved_id, |chat| {
                    chat.update_markdown(|x| {
                        x.remove(x.len() - 1);
                    });
                });

                if let Some(chat) = app.main_view.chats().get(&id) {
                    let option = app
                        .options
                        .get_create_model_options_index(chat.model().to_string());

                    app.main_view.add_chat_stream(
                        saved_id,
                        crate::llm::ChatStream::new(app, saved_id, option),
                    );
                }

                Task::none()
            }
            Self::Listen => {
                let mic = MicInput::default();
                let stream = mic.stream();

                app.main_view.update_chat(&id, |chat| {
                    if let Some(chat) = chat {
                        chat.set_state(State::Listening);
                    }
                });

                Task::perform(get_audio(stream), move |x| {
                    Message::Chats(ChatsMessage::Convert(x), id)
                })
            }
            Self::Convert(x) => {
                app.main_view.update_chat(&id, |chat| {
                    if let Some(chat) = chat {
                        chat.set_state(State::Generating);
                    }
                });

                Task::perform(transcribe(x.clone()), move |x| {
                    Message::Chats(ChatsMessage::Listened(x), id)
                })
            }
            Self::Listened(x) => {
                app.main_view.update_chat(&id, |chat| {
                    if let Some(chat) = chat {
                        if let Ok(str) = x {
                            chat.set_content_text(str);
                        }

                        chat.set_state(State::Idle);
                    }
                });

                Task::none()
            }
            Self::RemoveImage(x) => {
                app.main_view.update_chat(&id, |chat| {
                    if let Some(chat) = chat {
                        if let Ok(x) = chat.images().binary_search(&x) {
                            chat.images_mut().remove(x);
                        }
                    }
                });

                Task::none()
            }
            Self::PickedImage(x) => {
                if let Ok(x) = x {
                    let mut x = x.clone();
                    app.main_view.update_chat(&id, |chat| {
                        if let Some(chat) = chat {
                            chat.add_images(&mut x)
                        }
                    });
                }
                Task::none()
            }
            Self::ChangeModel(x) => {
                app.main_view.update_chat(&id, |chat| {
                    if let Some(chat) = chat {
                        if chat.state() == &State::Idle {
                            chat.set_model(x.clone());
                            app.save.save(SAVE_FILE);
                            let _ = app.options.get_create_model_options_index(x.clone());
                        }
                    }
                });

                Task::none()
            }
            Self::ChangeStart(x) => {
                app.main_view.update_chat(&id, |chat| {
                    if let Some(chat) = chat {
                        chat.set_start(x.clone());
                    }
                });
                Task::none()
            }
            Self::ChangeChat(x) => {
                app.main_view.update_chat(&id, |chat| {
                    if let Some(chat) = chat {
                        if chat.state() == &State::Idle {
                            chat.set_saved_chat(x.clone());
                            chat.set_markdown(app.save.chats.get(x).unwrap().to_mk());
                            app.save.save(SAVE_FILE);
                        }
                    }
                });

                Task::none()
            }
            Self::Action(x) => {
                app.main_view.update_chat(&id, |chat| {
                    if let Some(chat) = chat {
                        chat.content_perform(x.clone())
                    }
                });
                Task::none()
            }
            Self::NewChat => {
                if let Some(chat) = app.main_view.chats().get(&id) {
                    if chat.state() == &State::Idle {
                        return Self::new_chat(app, id);
                    }
                }

                Task::none()
            }
            Self::Submit => {
                let (chat, saved_id, option) = if let Some(chat) = app.main_view.chats().get(&id) {
                    (
                        ChatBuilder::default()
                            .content(chat.get_content_text())
                            .images(chat.images().clone())
                            .build()
                            .unwrap(),
                        chat.saved_chat().clone(),
                        app.options
                            .get_create_model_options_index(chat.model().to_string()),
                    )
                } else {
                    return Task::none();
                };

                let mut tools = &Vec::new();
                if let Some(x) = app.save.chats.get_mut(&saved_id) {
                    x.0.push(chat.clone());
                    tools = &x.1;
                }

                app.main_view.update_chat_by_saved(&saved_id, |x| {
                    x.add_markdown(Chat::generate_mk(chat.content()));
                });

                if tools.is_empty() {
                    app.main_view.add_chat_stream(
                        saved_id,
                        crate::llm::ChatStream::new(app, saved_id, option),
                    );
                }

                app.main_view.update_chat(&id, |chat| {
                    if let Some(chat) = chat {
                        if let Some(saved_chat) = app.save.chats.get(&saved_id) {
                            if saved_chat.1.is_empty() {
                                chat.set_state(State::Generating);
                            } else {
                                let tooled = TooledOptions {
                                    chats: saved_chat.get_chat_messages(),
                                    // tools : app.save.chats[s_index].2.clone(),
                                    tools: vec![Tools::DuckDuckGo],
                                };

                                chat.set_tools(Arc::new(tooled));

                                // return Task::perform(run_ollama_tools(app.main_view.chats[index].tools.clone(), app.options.0[option].clone(), app.logic.ollama.clone()), |x| Message::None)
                            }

                            chat.set_images(Vec::new());
                        }
                    }
                });

                Task::none()
            }
            Self::PickImage => ChatApp::pick_images(id),
        }
    }
}

impl SavedChats {
    pub fn new() -> Self {
        Self(Vec::new(), Vec::new(), SystemTime::now())
    }

    pub fn to_mk(&self) -> Vec<Vec<markdown::Item>> {
        return self
            .0
            .iter()
            .map(|x| Chat::generate_mk(&x.content()))
            .collect();
    }

    pub fn new_with_chats_tools(chats: Vec<Chat>, tools: Vec<Tools>) -> Self {
        return Self(chats, tools, SystemTime::now());
    }

    pub fn new_with_chats(chats: Vec<Chat>) -> Self {
        return Self(chats, Vec::new(), SystemTime::now());
    }

    pub fn get_preview(&self) -> (String, SystemTime) {
        return get_preview(self);
    }

    pub fn get_chat_messages(&self) -> Vec<ChatMessage> {
        self.0.iter().map(|x| x.into()).collect()
    }
}
