use super::{
    chat::{Chat, ChatBuilder, Role},
    tree::{ChatNode, Reason},
    view::State,
    SavedChat, TooledOptions, CHATS_FILE,
};
#[cfg(feature = "voice")]
use crate::sound::{get_audio, transcribe};
use crate::{
    common::Id,
    llm::{run_ollama_multi, ChatStreamId, Tools},
    prompts::view::get_command_input,
    ChatApp, Message,
};
use cli_clipboard::{ClipboardContext, ClipboardProvider};
use iced::{widget::text_editor, Task};
#[cfg(feature = "voice")]
use kalosm_sound::MicInput;
#[cfg(feature = "voice")]
use rodio::buffer::SamplesBuffer;
use std::{path::PathBuf, sync::Arc};

#[derive(Debug, Clone)]
pub enum ChatsMessage {
    PickedPrompt(String),
    SetPrompt(Option<String>, String),
    ChangePrompt(text_editor::Motion),
    SubmitPrompt,
    Regenerate(usize),
    Branch(usize),
    SaveEdit,
    CancelEdit,
    Edit(String),
    Submit,
    ChangeModel(usize, String),
    RemoveModel(usize),
    AddModel,
    Action(text_editor::Action),
    EditAction(text_editor::Action),
    ChangeStart(String),
    ChangeChat(Id),
    NewChat,
    #[cfg(feature = "voice")]
    Listen,
    #[cfg(feature = "voice")]
    Convert(Option<SamplesBuffer<f32>>),
    Listened(Result<String, String>),
    PickedImage(Result<Vec<PathBuf>, String>),
    ChangePath(usize, bool),
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
        let get_saved_id = || -> Option<Id> {
            if let Some(chat) = app.main_view.chats().get(&id) {
                Some(chat.saved_chat().clone())
            } else {
                None
            }
        };

        match self {
            Self::Branch(index) => {
                let saved_id = match get_saved_id() {
                    Some(x) => x,
                    None => return Task::none(),
                };

                let new_saved_id = Id::new();

                let mut new_saved = match app.chats.0.get(&saved_id) {
                    Some(x) => x.chats.get_full_history(),
                    None => return Task::none(),
                };

                if let Some(node) = new_saved.get(*index) {
                    if node.role() == &Role::AI {
                        let (n, _) = new_saved.split_at(*index);
                        new_saved = n.to_vec();
                    } else {
                        let (n, _) = new_saved.split_at(*index + 1);
                        new_saved = n.to_vec();
                    }
                };

                let new_saved = SavedChat::new_with_chats(new_saved.into());

                app.chats.0.insert(new_saved_id.clone(), new_saved);
                Self::changed_saved(app, id, new_saved_id);
                Task::none()
            }
            Self::Regenerate(index) => {
                let saved_id = match get_saved_id() {
                    Some(x) => x,
                    None => return Task::none(),
                };

                let mut parent_index = 0;
                let mut child_index = 0;

                if let Some(chat) = app.chats.0.get_mut(&saved_id) {
                    let parent = if let Some(node) = chat.chats.get_node_mut_from_index(index - 1) {
                        if node.chat.role() == &super::chat::Role::AI {
                            parent_index = *index;
                            chat.chats.get_node_mut_from_index(*index)
                        } else {
                            parent_index = index - 1;
                            Some(node)
                        }
                    } else {
                        None
                    };

                    if let Some(node) = parent {
                        child_index = node.children.len();
                        node.selected_child_index = Some(node.children.len());

                        for child in node.children.iter_mut() {
                            if child.reason.is_none() {
                                child.reason = Some(Reason::Sibling);
                            }
                        }

                        node.add_chat(
                            ChatBuilder::default()
                                .content(String::new())
                                .role(super::chat::Role::AI)
                                .build()
                                .unwrap(),
                            Some(Reason::Regeneration),
                        );
                    }
                }

                app.main_view.update_chat_by_saved(&saved_id, |chat| {
                    chat.update_markdown(|x| {
                        x.remove(x.len() - 1);
                        x.push(Chat::generate_mk(""));
                    });
                });

                if let Some(chat) = app.main_view.chats().get(&id) {
                    let option = app
                        .options
                        .get_create_model_options_index(chat.models()[0].to_string());

                    let chat_stream_id = ChatStreamId::new(saved_id, parent_index, child_index);

                    app.main_view.add_chat_stream(
                        chat_stream_id,
                        crate::llm::ChatStream::new(app, chat_stream_id, option),
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

            Self::ChangePath(index, next) => {
                let saved_id = match get_saved_id() {
                    Some(x) => x,
                    None => return Task::none(),
                };
                let mut mk = Vec::new();

                if let Some(chat) = app.chats.0.get_mut(&saved_id) {
                    if let Some(parent) = chat.chats.get_node_mut_from_index(index - 1) {
                        let len = parent.children.len();
                        if let Some(selected) = parent.selected_child_index {
                            if selected >= (len - 1) && next.clone() {
                                parent.selected_child_index = Some(0);
                            } else if selected <= 0 && !next {
                                parent.selected_child_index = Some(len - 1);
                            } else if next.clone() {
                                parent.selected_child_index = Some(selected + 1);
                            } else if !next {
                                parent.selected_child_index = Some(selected - 1);
                            }
                        }
                    }

                    mk = chat.to_mk();
                }

                app.main_view.update_chat_by_saved(&saved_id, |x| {
                    x.set_markdown(mk.clone());
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
                let saved_id = match get_saved_id() {
                    Some(x) => x,
                    None => return Task::none(),
                };
                if let Some(index) = app
                    .chats
                    .0
                    .get(&saved_id)
                    .unwrap()
                    .chats
                    .into_iter()
                    .position(|x| x.chat.content() == m)
                {
                    app.main_view.update_edits(|edits| {
                        if let Some(_) = edits.get(&id) {
                            let _ = edits.remove(&id);
                        } else {
                            edits.insert(id, index);
                        }
                    });
                    app.main_view.update_chat(&id, |chat| {
                        if let Some(chat) = chat {
                            chat.set_edit(text_editor::Content::with_text(
                                app.chats
                                    .0
                                    .get(&saved_id)
                                    .unwrap()
                                    .chats
                                    .get_node_from_index(index)
                                    .unwrap()
                                    .chat
                                    .content(),
                            ));
                        }
                    });
                }

                Task::none()
            }
            Self::SaveEdit => {
                let saved_id = match get_saved_id() {
                    Some(x) => x,
                    None => return Task::none(),
                };
                let mut mk = Vec::new();

                if let Some(edit) = app.main_view.edits().get(&id) {
                    if let Some(chat) = app.chats.0.get_mut(&saved_id) {
                        if let Some(node) = chat.chats.get_node_mut_from_index(edit.clone()) {
                            node.chat
                                .set_content(app.main_view.chats().get(&id).unwrap().edit().text());
                        }
                        mk = chat.to_mk();
                        app.chats.save(CHATS_FILE);
                    }
                }

                app.main_view.update_chat_by_saved(&saved_id, |x| {
                    x.set_markdown(mk.clone());
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
            #[cfg(feature = "voice")]
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
            #[cfg(feature = "voice")]
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
            Self::ChangeModel(i, x) => {
                app.main_view.update_chat(&id, |chat| {
                    if let Some(chat) = chat {
                        if chat.state() == &State::Idle {
                            if let Some(model) = chat.models_mut().get_mut(*i) {
                                *model = x.clone()
                            }

                            let _ = app.options.get_create_model_options_index(x.clone());
                        }
                    }
                });

                Task::none()
            }
            Self::RemoveModel(i) => {
                app.main_view.update_chat(&id, |chat| {
                    if let Some(chat) = chat {
                        if chat.state() == &State::Idle && chat.models().len() > 1 {
                            chat.models_mut().remove(*i);
                        }
                    }
                });

                Task::none()
            }
            Self::AddModel => {
                app.main_view.update_chat(&id, |chat| {
                    if let Some(chat) = chat {
                        if chat.state() == &State::Idle {
                            let model = chat
                                .models()
                                .first()
                                .unwrap_or(app.logic.models.first().unwrap())
                                .to_string();
                            chat.models_mut().push(model);
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
                Self::changed_saved(app, id, *x);
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
                let (chat, saved_id, option, models) =
                    if let Some(chat) = app.main_view.chats().get(&id) {
                        (
                            ChatBuilder::default()
                                .content(chat.get_content_text())
                                .images(chat.images().clone())
                                .role(super::chat::Role::User)
                                .build()
                                .unwrap(),
                            chat.saved_chat().clone(),
                            app.options
                                .get_create_model_options_index(chat.models()[0].to_string()),
                            chat.models().clone(),
                        )
                    } else {
                        return Task::none();
                    };

                let mut parent_index = 0;

                app.main_view.update_chat_by_saved(&saved_id, |x| {
                    if parent_index <= 0 {
                        parent_index = x.markdown().len();
                    }
                    x.add_markdown(Chat::generate_mk(chat.content()));
                    x.add_markdown(Chat::generate_mk(""));
                    x.set_state(State::Generating);
                });

                let mut tools = Vec::new();

                if let Some(x) = app.chats.0.get_mut(&saved_id) {
                    // parent_index = x.chats.get_full_path().len();
                    if let Some(parent) = x.chats.get_last_mut() {
                        parent.add_chat(chat.clone(), None);
                        let index = parent.children.len() - 1;
                        parent.selected_child_index = Some(index);
                        if let Some(parent) = parent.children.get_mut(index) {
                            if models.len() <= 1 {
                                parent.add_chat(
                                    ChatBuilder::default()
                                        .content(String::new())
                                        .role(super::chat::Role::AI)
                                        .build()
                                        .unwrap(),
                                    None,
                                );
                            } else {
                                for model in &models {
                                    parent.add_chat(
                                        ChatBuilder::default()
                                            .content(String::new())
                                            .role(super::chat::Role::AI)
                                            .build()
                                            .unwrap(),
                                        Some(Reason::Model(model.to_string())),
                                    );
                                }
                            }
                        };
                    }

                    tools = x.tools.clone();
                }

                for (i, _) in models.iter().enumerate() {
                    if tools.is_empty() {
                        let chat_stream_id = ChatStreamId::new(saved_id, parent_index, i);
                        app.main_view.add_chat_stream(
                            chat_stream_id,
                            crate::llm::ChatStream::new(app, chat_stream_id, option),
                        );
                    }
                }

                Task::none()
            }
            Self::PickImage => ChatApp::pick_images(id),
        }
    }
}
