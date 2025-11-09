use super::{
    chat::{Chat, ChatBuilder, Role},
    view::State,
    SavedChat, CHATS_FILE,
};
#[cfg(feature = "voice")]
use crate::sound::{get_audio, transcribe};
use crate::{
    chats::{chat::FileType, Reason, Relationship},
    common::Id,
    llm::ChatStreamId,
    prompts::view::get_command_input,
    ChatApp, Message,
};
use clipboard_rs::{Clipboard, ClipboardContext};
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
    SaveEdit(usize),
    CancelEdit,
    Edit(usize),
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
            if let Ok(clip_ctx) = ClipboardContext::new() {
                clip_ctx.get_text().unwrap_or(String::new())
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

                let path = app.main_view.chats().get(&id).unwrap().chats();

                let new_saved_id = Id::new();

                let (mut new_saved, index, tools): (Vec<Chat>, usize, Vec<Id>) =
                    match app.chats.0.get(&saved_id) {
                        Some(x) => {
                            let chats: Vec<(usize, Chat)> = x
                                .get_chats_with_reason(&path)
                                .into_iter()
                                .map(|x| (x.0, x.1.clone()))
                                .collect();

                            let index = chats
                                .iter()
                                .enumerate()
                                .find(|x| &x.1 .0 == index)
                                .map(|x| x.0)
                                .unwrap();
                            (
                                chats.into_iter().map(|x| x.1).collect(),
                                index,
                                x.default_tools.clone(),
                            )
                        }
                        None => return Task::none(),
                    };

                if new_saved[index].role() == &Role::AI {
                    new_saved = new_saved[0..=index].to_vec();
                } else {
                    new_saved = new_saved[0..=(index + 1)].to_vec();
                }

                let new_saved = SavedChat::new_with_chats(new_saved, tools);

                app.chats.0.insert(new_saved_id.clone(), new_saved);
                Self::changed_saved(app, id, new_saved_id);
                app.regenerate_side_chats(vec![id, new_saved_id])
            }
            Self::Regenerate(index) => {
                let saved_id = match get_saved_id() {
                    Some(x) => x,
                    None => return Task::none(),
                };

                let child_index = if let Some(chat) = app.chats.0.get_mut(&saved_id) {
                    let parent =
                        if chat.chats.chats.get(*index).unwrap().role() == &super::chat::Role::AI {
                            chat.get_parent_index(&index)
                        } else {
                            Some(*index)
                        };

                    let child = if let Some(parent) = parent {
                        chat.chats.chats.push(
                            ChatBuilder::default()
                                .content(String::new())
                                .role(super::chat::Role::AI)
                                .build()
                                .unwrap(),
                        );
                        let child = chat.chats.chats.len() - 1;

                        *app.main_view
                            .chats_mut()
                            .get_mut(&id)
                            .unwrap()
                            .chats_mut()
                            .iter_mut()
                            .find(|x| x == &index)
                            .unwrap() = child;

                        if let Some(relationship) = chat.chats.relationships.get_mut(&parent) {
                            for child in relationship.iter_mut() {
                                if child.reason.is_none() {
                                    child.reason = Some(Reason::Sibling);
                                }
                            }
                            relationship.push(Relationship {
                                index: child,
                                reason: Some(Reason::Regeneration),
                            });
                        }

                        Some(child)
                    } else {
                        None
                    };
                    child
                } else {
                    None
                };

                let (chats, before, old_tools) =
                    if let Some(chat) = app.main_view.chats_mut().get_mut(&id) {
                        let index = chat.chats().iter().find(|x| x == &index).unwrap().clone();
                        chat.update_markdown(|x| x[index] = Chat::generate_mk(""));
                        (chat.chats().clone(), index, chat.tools().clone())
                    } else {
                        (Vec::new(), 0, Vec::new())
                    };

                let chats = if let Some(chat) = app.chats.0.get(&saved_id) {
                    chat.get_chat_messages_before(&chats, before)
                } else {
                    Vec::new()
                };

                let mut tools = Vec::new();

                for tool in &old_tools {
                    if let Some(tool) = app.tools.tools.get(tool) {
                        tools.push(tool.clone());
                    }
                }

                let (chats, tools) = (Arc::new(chats), Arc::new(tools));

                if let Some(chat) = app.main_view.chats().get(&id) {
                    let option = app
                        .options
                        .get_create_model_options_index(chat.models()[0].to_string());

                    let chat_stream_id = ChatStreamId::new(saved_id, child_index.unwrap());

                    app.main_view.add_chat_stream(
                        chat_stream_id,
                        crate::llm::ChatStream::new(
                            app,
                            chats,
                            tools,
                            option,
                            app.logic.get_random_provider().unwrap(),
                        ),
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

                let (child, mut new_path) = if let Some(chat) = app.chats.0.get_mut(&saved_id) {
                    let parent = chat.get_parent_index(index).unwrap();
                    if let Some(relationship) = chat.chats.relationships.get(&parent) {
                        let len = relationship.len();
                        let selected = relationship
                            .iter()
                            .enumerate()
                            .find(|x| &x.1.index == index)
                            .map(|x| x.0)
                            .unwrap();

                        let index = if selected >= (len - 1) && next.clone() {
                            0
                        } else if selected <= 0 && !next {
                            len - 1
                        } else if next.clone() {
                            selected + 1
                        } else {
                            selected - 1
                        };

                        let i = relationship.get(index).unwrap().index;
                        let path = chat.get_path_from_index(i);

                        (i, path)
                    } else {
                        return Task::none();
                    }
                } else {
                    return Task::none();
                };

                if let Some(chat) = app.main_view.chats_mut().get_mut(&id) {
                    let og_index = chat
                        .chats()
                        .iter()
                        .enumerate()
                        .find(|(_, x)| x == &index)
                        .map(|(i, _)| i)
                        .unwrap();
                    let mut path = chat.chats()[0..og_index].to_vec();
                    path.push(child);
                    path.append(&mut new_path);

                    *chat.chats_mut() = path.clone();
                    let mk = app.chats.0.get(&saved_id).unwrap().to_mk(&path);
                    app.chats.0.get_mut(&saved_id).unwrap().default_chats = path;
                    chat.set_markdown(mk);
                }

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
            Self::Edit(index) => {
                let saved_id = match get_saved_id() {
                    Some(x) => x,
                    None => return Task::none(),
                };

                app.main_view.update_chat(&id, |chat| {
                    if let Some(chat) = chat {
                        if let Some(_) = chat.edit_index() {
                            *chat.edit_index_mut() = None;
                        } else {
                            *chat.edit_index_mut() = Some(*index);
                            chat.set_edit(text_editor::Content::with_text(
                                app.chats
                                    .0
                                    .get(&saved_id)
                                    .unwrap()
                                    .chats
                                    .chats
                                    .get(*index)
                                    .unwrap()
                                    .content(),
                            ));
                        }
                    }
                });

                Task::none()
            }
            Self::SaveEdit(index) => {
                let saved_id = match get_saved_id() {
                    Some(x) => x,
                    None => return Task::none(),
                };

                if let Some(chat) = app.chats.0.get_mut(&saved_id) {
                    if let Some(node) = chat.chats.chats.get_mut(*index) {
                        node.set_content(app.main_view.chats().get(&id).unwrap().edit().text());
                    }
                    app.chats.save(CHATS_FILE);
                }

                if let Some(chat) = app.main_view.chats_mut().get_mut(&id) {
                    *chat.edit_index_mut() = None;
                    chat.set_markdown(app.chats.0.get(&saved_id).unwrap().to_mk(&chat.chats()));
                }

                Task::none()
            }
            Self::CancelEdit => {
                *app.main_view
                    .chats_mut()
                    .get_mut(&id)
                    .unwrap()
                    .edit_index_mut() = None;
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
                            if !chat.models().is_empty() || !app.logic.models.is_empty() {
                                let model = chat
                                    .models()
                                    .first()
                                    .unwrap_or(app.logic.models.first().unwrap())
                                    .to_string();
                                chat.models_mut().push(model);
                            }
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
            Self::NewChat => Self::new_chat(app, id),
            Self::Submit => {
                let (chat, saved_id, options, models, old_tools, mut chats) =
                    if let Some(chat) = app.main_view.chats_mut().get_mut(&id) {
                        let mut options = Vec::new();
                        let models = chat.models().clone();

                        for model in &models {
                            options.push(
                                app.options
                                    .get_create_model_options_index(model.to_string()),
                            );
                        }

                        let submission = ChatBuilder::default()
                            .content(chat.get_content_text())
                            .images(
                                chat.images()
                                    .iter()
                                    .map(|x| FileType::Path(x.clone()))
                                    .collect(),
                            )
                            .role(super::chat::Role::User)
                            .build()
                            .unwrap();

                        (
                            submission,
                            chat.saved_chat().clone(),
                            options,
                            models,
                            chat.tools().clone(),
                            chat.chats().clone(),
                        )
                    } else {
                        return Task::none();
                    };

                let last = if chats.len() > 0 { chats.len() - 1 } else { 0 };

                let (mut chats, parent_index) = if let Some(x) = app.chats.0.get_mut(&saved_id) {
                    let parent = {
                        x.chats.chats.push(chat.clone());
                        let parent = x.chats.chats.len() - 1;

                        if let Some(og_parent) = chats.last() {
                            x.chats.relationships.insert(
                                *og_parent,
                                vec![Relationship {
                                    index: parent,
                                    reason: None,
                                }],
                            );
                        }

                        chats.push(parent);
                        chats.push(parent + 1);

                        if models.len() <= 1 {
                            x.chats.chats.push(
                                ChatBuilder::default()
                                    .content(String::new())
                                    .role(super::chat::Role::AI)
                                    .build()
                                    .unwrap(),
                            );
                            let index = x.chats.chats.len() - 1;

                            x.chats.relationships.insert(
                                parent,
                                vec![Relationship {
                                    index,
                                    reason: None,
                                }],
                            );
                        } else {
                            let mut relationships = Vec::new();

                            for model in &models {
                                x.chats.chats.push(
                                    ChatBuilder::default()
                                        .content(String::new())
                                        .role(super::chat::Role::AI)
                                        .build()
                                        .unwrap(),
                                );
                                let index = x.chats.chats.len() - 1;

                                relationships.push(Relationship {
                                    index,
                                    reason: Some(Reason::Model(model.to_string())),
                                });
                            }

                            x.chats.relationships.insert(parent, relationships);
                        }
                        parent
                    };
                    if !x.default_chats.contains(chats.last().unwrap()) {
                        x.default_chats = chats.clone();

                        if chats.len() > 2 {
                            app.main_view.update_chat_by_saved_and_message(
                                &saved_id,
                                &chats[last],
                                |c| {
                                    c.add_markdown(Chat::generate_mk(chat.content()));
                                    c.add_markdown(Chat::generate_mk(""));
                                    c.set_state(State::Generating);
                                    *c.chats_mut() = chats.clone();
                                },
                            );
                        }
                        if let Some(c) = app.main_view.chats_mut().get_mut(&id) {
                            c.add_markdown(Chat::generate_mk(chat.content()));
                            c.add_markdown(Chat::generate_mk(""));
                            c.set_state(State::Generating);
                            *c.chats_mut() = chats.clone();
                        };
                    }

                    (x.get_chat_messages(&chats), parent)
                } else {
                    (Vec::new(), 0)
                };

                let mut tools = Vec::new();

                for tool in &old_tools {
                    if let Some(tool) = app.tools.tools.get(tool) {
                        tools.push(tool.clone());
                    }
                }

                chats.pop();

                let (chats, tools) = (Arc::new(chats), Arc::new(tools));

                for (i, option) in options.into_iter().enumerate() {
                    if tools.is_empty() {
                        let chat_stream_id = ChatStreamId::new(saved_id, parent_index + i + 1);
                        app.main_view.add_chat_stream(
                            chat_stream_id,
                            crate::llm::ChatStream::new(
                                app,
                                chats.clone(),
                                tools.clone(),
                                option,
                                app.logic.get_random_provider().unwrap(),
                            ),
                        );
                    }
                }

                Task::none()
            }
            Self::PickImage => ChatApp::pick_images(id),
        }
    }
}
