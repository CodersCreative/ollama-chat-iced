use std::{path::PathBuf, sync::Arc};

use cli_clipboard::{ClipboardContext, ClipboardProvider};
use iced::{widget::text_editor, Task};
use kalosm_sound::MicInput;
use rodio::buffer::SamplesBuffer;

use crate::{
    common::Id,
    llm::Tools,
    prompts::view::get_command_input,
    sound::{get_audio, transcribe},
    ChatApp, Message,
};

use super::{
    chat::{Chat, ChatBuilder},
    view::State,
    TooledOptions, CHATS_FILE,
};

#[derive(Debug, Clone)]
pub enum ChatsMessage {
    PickedPrompt(String),
    SetPrompt(Option<String>, String),
    ChangePrompt(text_editor::Motion),
    SubmitPrompt,
    Regenerate,
    SaveEdit,
    CancelEdit,
    Edit(String),
    Submit,
    ChangeModel(String),
    Action(text_editor::Action),
    EditAction(text_editor::Action),
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
    fn set_picked_prompt(app: &mut ChatApp, x: &str, id: &Id) {
        let clip = {
            if let Ok(mut clip_ctx) = ClipboardContext::new() {
                clip_ctx.get_contents().unwrap_or(String::new())
            } else {
                String::new()
            }
        };

        app.main_view.update_chat(&id, |chat| {
            if let Some(chat) = chat {
                if let Some(command) = app.prompts.prompts.iter().find(|y| &y.1.command == x) {
                    chat.set_content(text_editor::Content::with_text(
                        &command.1.content.replace("{{CLIPBOARD}}", clip.as_str()),
                    ));
                }
                chat.set_selected_prompt(None);
            }
        });
    }

    pub fn handle(&self, id: Id, app: &mut ChatApp) -> Task<Message> {
        match self {
            Self::Regenerate => {
                let saved_id = app.main_view.chats().get(&id).unwrap().saved_chat().clone();

                for x in app.chats.0.iter_mut().filter(|x| x.0 == &saved_id) {
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
            Self::PickedPrompt(x) => {
                Self::set_picked_prompt(app, x, &id);
                Task::none()
            }
            Self::SetPrompt(c, x) => {
                if let Some(c) = c {
                    app.main_view.update_chat(&id, |chat| {
                        if let Some(chat) = chat {
                            if let Some(command) =
                                app.prompts.prompts.iter().find(|y| &y.1.command == x)
                            {
                                chat.set_content(text_editor::Content::with_text(
                                    &command.1.content.replace("{{clipboard}}", c.as_str()),
                                ));
                            }
                        }
                    });
                }
                Task::none()
            }
            Self::ChangePrompt(x) => {
                app.main_view.update_chat(&id, |chat| {
                    if let Some(chat) = chat {
                        if let Some(command_input) = get_command_input(&chat.content().text()) {
                            if let Ok(prompts) = app.prompts.search(command_input) {
                                match x {
                                    text_editor::Motion::Up => {
                                        if let Some(selected) = chat.selected_prompt() {
                                            if selected > &0 {
                                                chat.set_selected_prompt(Some(
                                                    selected.clone() - 1,
                                                ));
                                            } else if selected >= &prompts.len() {
                                                chat.set_selected_prompt(Some(0));
                                            } else {
                                                chat.set_selected_prompt(Some(prompts.len() - 1));
                                            }
                                        } else if prompts.len() > 0 {
                                            chat.set_selected_prompt(Some(prompts.len() - 1));
                                        }
                                    }
                                    _ => {
                                        if let Some(selected) = chat.selected_prompt() {
                                            if selected < &(prompts.len() - 2) {
                                                chat.set_selected_prompt(Some(
                                                    selected.clone() + 1,
                                                ));
                                            } else {
                                                chat.set_selected_prompt(Some(0));
                                            }
                                        } else if prompts.len() > 0 {
                                            chat.set_selected_prompt(Some(0));
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
                Task::none()
            }
            Self::SubmitPrompt => {
                if let Some(chat) = app.main_view.chats().get(&id) {
                    if let Some(command_input) = get_command_input(&chat.content().text()) {
                        if let Ok(prompts) = app.prompts.search(command_input) {
                            if let Some(selected) = chat.selected_prompt() {
                                Self::set_picked_prompt(
                                    app,
                                    &prompts[selected.clone()].command,
                                    &id,
                                );
                            }
                        }
                    }
                }
                Task::none()
            }
            Self::Edit(m) => {
                let saved_id = app.main_view.chats().get(&id).unwrap().saved_chat().clone();
                if let Some(index) = app
                    .chats
                    .0
                    .get(&saved_id)
                    .unwrap()
                    .0
                    .iter()
                    .position(|x| x.content() == m)
                {
                    app.main_view.update_edits(|edits| {
                        edits.insert(id, index);
                    });
                    app.main_view.update_chat(&id, |chat| {
                        chat.unwrap().set_edit(text_editor::Content::with_text(
                            app.chats.0.get(&saved_id).unwrap().0[index].content(),
                        ));
                    });
                }

                Task::none()
            }
            Self::SaveEdit => {
                let saved_id = app.main_view.chats().get(&id).unwrap().saved_chat().clone();
                let mut mk = Vec::new();

                if let Some(edit) = app.main_view.edits().get(&id) {
                    if let Some(chat) = app.chats.0.get_mut(&saved_id) {
                        chat.0[edit.clone()]
                            .set_content(app.main_view.chats().get(&id).unwrap().edit().text());
                        mk = chat.to_mk();
                        app.chats.save(CHATS_FILE);
                    }
                }

                app.main_view.update_chat_by_saved(&saved_id, |x| {
                    x.set_markdown(mk.clone());
                    // x.add_markdown(Chat::generate_mk(chat.content()));
                });
                app.main_view.update_edits(|edits| {
                    edits.remove(&id);
                });
                Task::none()
            }
            Self::CancelEdit => {
                app.main_view.update_edits(|edits| {
                    edits.remove(&id);
                });
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
                            app.chats.save(CHATS_FILE);
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
                            chat.set_markdown(app.chats.0.get(x).unwrap().to_mk());
                            app.chats.save(CHATS_FILE);
                        }
                    }
                });

                Task::none()
            }
            Self::Action(x) => {
                app.main_view.update_chat(&id, |chat| {
                    if let Some(chat) = chat {
                        chat.content_perform(x.clone());
                        if let Some(command_input) = get_command_input(&chat.content().text()) {
                            if let Ok(_) = app.prompts.search(command_input) {
                                return;
                            }
                        }
                        chat.set_selected_prompt(None);
                    }
                });
                Task::none()
            }
            Self::EditAction(x) => {
                app.main_view.update_chat(&id, |chat| {
                    if let Some(chat) = chat {
                        chat.edit_mut().perform(x.clone());
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
                if let Some(x) = app.chats.0.get_mut(&saved_id) {
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
                        if let Some(saved_chat) = app.chats.0.get(&saved_id) {
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
