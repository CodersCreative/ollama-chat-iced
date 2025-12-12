use std::collections::HashMap;

use iced::{
    alignment::Vertical, clipboard, widget::{
        button, column, container, mouse_area, pick_list, row, scrollable, space, text, text_editor
    }, Element, Length, Padding, Task
};
use ochat_types::{
    chats::{
        Chat,
        messages::{MessageData, MessageDataBuilder, Role},
    },
    generation::text::{ChatQueryData, ChatQueryMessage},
    settings::SettingsProvider,
};

use crate::{
    data::RequestType, font::{BODY_SIZE, SMALL_SIZE}, pages::home::panes::{
        data::{MessageMk, PromptsData},
        view::HomePaneViewMessage,
    }, style, subscriptions::SubMessage, Application, CacheMessage, Message, DATA
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
    CancelGenerating,
    UploadFile,
    UserMessageUploaded(MessageMk),
    AIMessageUploaded(MessageMk, Option<ChatQueryData>),
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
                let search = get_command_input(&view.input.text())
                    .unwrap_or_default()
                    .to_string();

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
            Self::UploadFile => {
                // TODO
                Task::none()
            }
            Self::CancelGenerating => {
                let messages = app.get_chats_view(&id).unwrap().messages.clone();
                let ids: Vec<u32> = app
                    .subscriptions
                    .message_gens
                    .iter()
                    .filter(|x| messages.contains(&x.1.id))
                    .map(|x| x.0.clone())
                    .collect();

                Task::batch(
                    ids.into_iter()
                        .map(|x| Task::done(Message::Subscription(SubMessage::StopGenMessage(x)))),
                )
            }
            Self::SubmitInput => {
                let (user_message, parent, chat_id) = {
                    let view = app.get_chats_view(&id).unwrap();
                    (
                        MessageDataBuilder::default()
                            .content(view.input.text())
                            .role(Role::User)
                            .build()
                            .unwrap(),
                        view.messages.last().map(|x| x.to_string()),
                        view.chat.id.key().to_string(),
                    )
                };

                Task::future(async move {
                    let req = DATA.read().unwrap().to_request();
                    if let Ok(x) = req
                        .make_request::<ochat_types::chats::messages::Message, MessageData>(
                            &if let Some(parent) = &parent {
                                format!("message/parent/{}", parent)
                            } else {
                                "message/".to_string()
                            },
                            &user_message,
                            RequestType::Post,
                        )
                        .await
                    {
                        if parent.is_none() {
                            let _ = req
                                .make_request::<ochat_types::chats::messages::Message, MessageData>(
                                    &format!("chat/{}/root/{}", chat_id, x.id.key().to_string()),
                                    &user_message,
                                    RequestType::Put,
                                )
                                .await;
                        }
                        Message::HomePaneView(HomePaneViewMessage::Chats(
                            id,
                            ChatsViewMessage::UserMessageUploaded(MessageMk::get(x).await),
                        ))
                    } else {
                        Message::None
                    }
                })
            }
            Self::UserMessageUploaded(user_message) => {
                if let Some(x) = app
                    .get_chats_view(&id)
                    .unwrap()
                    .messages
                    .last()
                    .map(|x| x.clone())
                {
                    for view in app
                        .view_data
                        .home
                        .chats
                        .iter_mut()
                        .filter(|y| y.1.messages.contains(&x))
                    {
                        view.1.messages.push(user_message.base.id.key().to_string());
                    }
                } else {
                    // TODO Do a filter to find all empty chats that use the same chat id.
                    app.get_chats_view(&id)
                        .unwrap()
                        .messages
                        .push(user_message.base.id.key().to_string());
                }

                app.cache
                    .home_shared
                    .messages
                    .0
                    .insert(user_message.base.id.key().to_string(), user_message.clone());
                let messages = app.get_chats_view(&id).unwrap().messages.clone();

                let messages: Vec<ChatQueryMessage> = messages
                    .iter()
                    .map(|x| {
                        app.cache
                            .home_shared
                            .messages
                            .0
                            .get(x)
                            .unwrap()
                            .base
                            .clone()
                            .into()
                    })
                    .collect();

                let req = DATA.read().unwrap().to_request();

                Task::batch(app.get_chats_view(&id).unwrap().models.clone().into_iter().map(|x| {
                    let user_message = user_message.base.id.key().to_string();
                    let messages = messages.clone();
                    let req = req.clone();
                    let id = id.clone();
                    Task::future(async move {
                        let message = MessageDataBuilder::default()
                            .content(String::new())
                            .role(Role::AI)
                            .build()
                            .unwrap();
                        if let Ok(message) = req
                            .make_request::<ochat_types::chats::messages::Message, MessageData>(
                                &format!("message/parent/{}", user_message),
                                &message,
                                RequestType::Post,
                            )
                            .await
                        {
                            Message::HomePaneView(HomePaneViewMessage::Chats(
                            id,
                            ChatsViewMessage::AIMessageUploaded(MessageMk::get(message).await, Some(ChatQueryData { provider: x.provider, model: x.model, messages })),
                        ))                            
                        } else {
                            Message::None
                        }
                    })
                }))
            }
            Self::AIMessageUploaded(message, query) => {
                if let Some(x) = app
                    .get_chats_view(&id)
                    .unwrap()
                    .messages
                    .last()
                    .map(|x| x.clone())
                {
                    for view in app
                        .view_data
                        .home
                        .chats
                        .iter_mut()
                        .filter(|y| y.1.messages.contains(&x))
                    {
                        view.1.messages.push(message.base.id.key().to_string());
                    }
                }
                let id = message.base.id.key().to_string();
                app.cache
                    .home_shared
                    .messages
                    .0
                    .insert(id.clone(), message.clone());
                if let Some(query) = query {
                    Task::done(Message::Subscription(SubMessage::GenMessage(id, query)))
                } else {
                    Task::none()
                }
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

pub fn get_command_input(input: &str) -> Option<&str> {
    if let Some(split) = input.split_whitespace().last() {
        if split.contains("/") {
            return Some(split.trim_start_matches("/"));
        }
    }

    None
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

    pub fn view<'a>(&'a self, app: &'a Application, id: u32) -> Element<'a, Message> {
        let is_generating = app
            .subscriptions
            .message_gens
            .iter()
            .find(|x| self.messages.contains(&x.1.id))
            .is_some();

        let input: Element<Message> = if !is_generating {
            text_editor(&self.input)
                .placeholder("Type your message here...")
                .on_action(move |action| {
                    Message::HomePaneView(HomePaneViewMessage::Chats(
                        id,
                        ChatsViewMessage::InputAction(action),
                    ))
                })
                .padding(Padding::from(20))
                .size(20)
                .style(style::text_editor::input)
                .key_binding(move |key_press| {
                    let modifiers = key_press.modifiers;

                    let is_command = !self.prompts.0.is_empty();

                    Some(text_editor::Binding::Custom(Message::HomePaneView(
                        HomePaneViewMessage::Chats(
                            id,
                            match text_editor::Binding::from_key_press(key_press) {
                                Some(text_editor::Binding::Enter)
                                    if !modifiers.shift() && is_command =>
                                {
                                    ChatsViewMessage::ApplyPrompt(None)
                                }
                                Some(text_editor::Binding::Move(text_editor::Motion::Up))
                                    if !modifiers.shift() && is_command =>
                                {
                                    ChatsViewMessage::ChangePrompt(text_editor::Motion::Up)
                                }
                                Some(text_editor::Binding::Move(text_editor::Motion::Down))
                                    if !modifiers.shift() && is_command =>
                                {
                                    ChatsViewMessage::ChangePrompt(text_editor::Motion::Down)
                                }
                                Some(text_editor::Binding::Enter) if !modifiers.shift() => {
                                    ChatsViewMessage::SubmitInput
                                }
                                binding => return binding,
                            },
                        ),
                    )))
                })
                .into()
        } else {
            container(
                text("Awaiting Response...")
                    .color(app.theme().palette().primary)
                    .size(20),
            )
            .padding(20)
            .width(Length::Fill)
            .style(container::transparent)
            .into()
        };

        let btn = |file: &'static str| style::svg_button::primary(file, 48);

        let btn_small = |file: &'static str| style::svg_button::primary(file, BODY_SIZE);

        let upload = btn("upload.svg").on_press(Message::HomePaneView(HomePaneViewMessage::Chats(
            id,
            ChatsViewMessage::UploadFile,
        )));

        let submit: Element<Message> = match is_generating {
            true => btn("close.svg")
                .on_press(Message::HomePaneView(HomePaneViewMessage::Chats(
                    id,
                    ChatsViewMessage::CancelGenerating,
                )))
                .into(),
            false => btn("send.svg")
                .on_press(Message::HomePaneView(HomePaneViewMessage::Chats(
                    id,
                    ChatsViewMessage::SubmitInput,
                )))
                .into(),
        };

        let bottom = container(
            row![upload, input, submit]
                .align_y(Vertical::Center)
                .spacing(5),
        )
        .max_height(350);

        let input = container(column![
            self.view_commands(app, id.clone()),
            container(row![
                scrollable::Scrollable::new(row(self.models.clone().into_iter().enumerate().map(
                    |(i, model)| {
                        mouse_area(
                            pick_list(DATA.read().unwrap().models.clone(), Some(model), move |x| {
                                Message::HomePaneView(HomePaneViewMessage::Chats(
                                    id,
                                    ChatsViewMessage::ChangeModel(i, x),
                                ))
                            })
                            .style(style::pick_list::main)
                            .menu_style(style::menu::main)
                            .text_size(BODY_SIZE),
                        )
                        .on_right_press(Message::HomePaneView(HomePaneViewMessage::Chats(
                            id,
                            ChatsViewMessage::RemoveModel(i),
                        )))
                        .into()
                    }
                )).spacing(5))
                .width(Length::Fill).direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::new())),
                btn_small("add.svg").on_press(Message::HomePaneView(HomePaneViewMessage::Chats(
                    id,
                    ChatsViewMessage::AddModel,
                ))),
            ].spacing(10).align_y(Vertical::Center))
            .width(Length::Fill)
            .align_y(Vertical::Center)
            .style(style::container::bottom_input_back),
            bottom,
        ])
        .width(Length::FillPortion(10))
        .padding(Padding::from([10, 20]))
        .style(style::container::input_back);

        let input = container(input).padding(10);

        /*let body = match self.messages.is_empty() {
            true => self.view_start(app, id.clone()),
            false => self.view_chat(app, &id),
        };*/
        let body = text("Hello, World!");

        container(column![body, space::vertical(), input,])
            .width(Length::FillPortion(50))
            .into()
    }

    fn view_commands<'a>(&'a self, app: &'a Application, id: u32) -> Element<'a, Message> {
        container(
            scrollable::Scrollable::new(row(self.prompts.0.iter().map(|x| {
                let chosen = if let Some(y) = &self.selected_prompt {
                    y == &x.id.key().to_string()
                } else {
                    false
                };

                button(text(&x.command).size(SMALL_SIZE))
                    .width(Length::Fill)
                    .style(if chosen {
                        style::button::chosen_chat
                    } else {
                        style::button::not_chosen_chat
                    })
                    .padding(10)
                    .into()
            })))
            .width(Length::Fill),
        )
        .max_height(250)
        .into()
    }
}
