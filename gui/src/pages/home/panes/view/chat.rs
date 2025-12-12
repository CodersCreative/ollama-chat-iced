use std::collections::HashMap;

use iced::{Element, Task, clipboard, widget::text_editor};
use ochat_types::{chats::Chat, settings::SettingsProvider};

use crate::{
    Application, DATA, Message,
    pages::home::panes::{
        data::{MessageMk, PromptsData},
        view::HomePaneViewMessage,
    },
};

#[derive(Debug, Clone)]
pub struct ChatsView {
    pub input: text_editor::Content,
    pub models: Vec<SettingsProvider>,
    pub edits: HashMap<String, text_editor::Content>,
    pub expanded_messages: Vec<String>,
    pub prompts: PromptsData,
    pub selected_prompt: Option<String>,
    pub messages: Vec<String>,
    pub chat: Chat,
}

#[derive(Debug, Clone)]
pub enum ChatsViewMessage {
    SetPrompts(PromptsData),
    ApplyPrompt(Option<String>),
    ChangePrompt(text_editor::Motion),
    SetInput(text_editor::Content),
    InputAction(text_editor::Action),
    SubmitInput,
    Regenerate(String),
    Branch(String),
    Expand(String),
    Edit(String),
    EditAction(String, text_editor::Action),
    SubmitEdit(String),
    AddModel,
    ChangeModel(usize, SettingsProvider),
    RemoveModel(usize),
}

impl ChatsViewMessage {
    pub fn handle(self, app: &mut Application, id: u32) -> Task<Message> {
        match self {
            Self::SetInput(x) => {
                let view = app.get_chats_view(&id).unwrap();
                view.input = x;
                view.selected_prompt = None;
                view.prompts.0.clear();
                Task::none()
            }
            Self::InputAction(action) => {
                let view = app.get_chats_view(&id).unwrap();
                view.prompts.0.clear();
                view.selected_prompt = None;
                view.input.perform(action);
                let search = view.input.text();

                if search.is_empty() {
                    Task::none()
                } else {
                    Task::future(async move {
                        let prompts = PromptsData::get_prompts(Some(search)).await;

                        Message::HomePaneView(HomePaneViewMessage::Chats(
                            id,
                            ChatsViewMessage::SetPrompts(prompts),
                        ))
                    })
                }
            }
            Self::SetPrompts(x) => {
                app.get_chats_view(&id).unwrap().prompts = x;
                Task::none()
            }
            Self::ChangePrompt(motion) => {
                let view = app.get_chats_view(&id).unwrap();
                let index = if let Some(s) = view.selected_prompt.clone() {
                    view.prompts
                        .0
                        .iter()
                        .position(|x| x.id.key().to_string() == s)
                } else {
                    None
                };

                let index: Option<usize> = match motion {
                    text_editor::Motion::Up => {
                        if let Some(selected) = index {
                            if selected > 0 {
                                Some(selected.clone() - 1)
                            } else if selected >= view.prompts.0.len() {
                                Some(0)
                            } else {
                                Some(view.prompts.0.len() - 1)
                            }
                        } else if view.prompts.0.len() > 0 {
                            Some(view.prompts.0.len() - 1)
                        } else {
                            None
                        }
                    }
                    _ => {
                        if let Some(selected) = index {
                            if selected < (view.prompts.0.len() - 2) {
                                Some(selected.clone() + 1)
                            } else {
                                Some(0)
                            }
                        } else if view.prompts.0.len() > 0 {
                            Some(0)
                        } else {
                            None
                        }
                    }
                };

                if let Some(index) = index {
                    view.selected_prompt =
                        view.prompts.0.get(index).map(|x| x.id.key().to_string());
                } else {
                    view.selected_prompt = None;
                }

                Task::none()
            }
            Self::ApplyPrompt(x) => {
                let view = app.get_chats_view(&id).unwrap();
                let prompt = if let Some(x) = x {
                    x
                } else {
                    view.selected_prompt.clone().unwrap()
                };

                let prompt = view
                    .prompts
                    .0
                    .iter()
                    .find(|y| y.id.key().to_string() == prompt)
                    .map(|x| x.clone())
                    .unwrap();

                clipboard::read_primary().map(move |clip| {
                    Message::HomePaneView(HomePaneViewMessage::Chats(
                        id,
                        ChatsViewMessage::SetInput(text_editor::Content::with_text(
                            &prompt
                                .content
                                .replace("{{CLIPBOARD}}", &clip.unwrap_or_default()),
                        )),
                    ))
                })
            }
            Self::SubmitInput => {
                // TODO
                Task::none()
            }
            Self::Expand(x) => {
                let view = app.get_chats_view(&id).unwrap();

                if view.expanded_messages.contains(&x) {
                    let _ = view.expanded_messages.retain(|y| y != &x);
                } else {
                    let _ = view.expanded_messages.push(x);
                }

                Task::none()
            }
            Self::Edit(x) => {
                let text = app
                    .cache
                    .home_shared
                    .messages
                    .0
                    .get(&x)
                    .unwrap()
                    .base
                    .content
                    .clone();

                let view = app.get_chats_view(&id).unwrap();

                if view.edits.contains_key(&x) {
                    let _ = view.edits.remove(&x);
                } else {
                    let _ = view.edits.insert(x, text_editor::Content::with_text(&text));
                }

                Task::none()
            }
            Self::EditAction(x, action) => {
                if let Some(msg) = app.get_chats_view(&id).unwrap().edits.get_mut(&x) {
                    msg.perform(action);
                }
                Task::none()
            }
            Self::SubmitEdit(_) => {
                // TODO
                Task::none()
            }
            Self::Regenerate(_) => {
                // TODO
                Task::none()
            }
            Self::Branch(_) => {
                // TODO
                Task::none()
            }
            Self::AddModel => {
                if let Some(model) = match app.cache.client_settings.default_provider.clone() {
                    Some(x) => Some(x),
                    _ => DATA.read().unwrap().models.first().map(|x| x.clone()),
                } {
                    app.get_chats_view(&id).unwrap().models.push(model);
                }
                Task::none()
            }
            Self::ChangeModel(index, model) => {
                *app.get_chats_view(&id)
                    .unwrap()
                    .models
                    .get_mut(index)
                    .unwrap() = model;
                Task::none()
            }
            Self::RemoveModel(index) => {
                let view = app.get_chats_view(&id).unwrap();

                if view.models.len() > 1 {
                    let _ = view.models.remove(index);
                }
                Task::none()
            }
        }
    }
}

impl ChatsView {
    pub fn view_message<'a>(
        _id: u32,
        _message: &'a MessageMk,
        _edit: &'a text_editor::Content,
        _expanded: bool,
    ) -> Element<'a, Message> {
        todo!()
    }

    pub fn view<'a>(&'a self, _app: &'a Application, _id: u32) -> Element<'a, Message> {
        todo!()
    }
}
