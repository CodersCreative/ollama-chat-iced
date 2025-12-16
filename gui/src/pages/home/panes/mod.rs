use crate::{
    Application, CacheMessage, DATA, Message,
    data::RequestType,
    pages::{
        PageMessage, Pages,
        home::{
            HomePage,
            message::{HomeMessage, HomePickingType},
            panes::{
                data::{MessageMk, MessagesData, PromptsData},
                view::{
                    chat::ChatsView, models::ModelsView, options::OptionsView,
                    prompts::PromptsView, pulls::PullsView, settings::SettingsView,
                },
            },
            sidebar::PreviewMk,
        },
    },
    windows::message::WindowMessage,
};
use iced::{
    Task,
    widget::{pane_grid, text_editor},
    window,
};
use ochat_types::chats::{Chat, ChatData, previews::Preview};
use std::collections::HashMap;

pub mod data;
pub mod view;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HomePaneType {
    Loading,
    Chat,
    Pulls,
    Models,
    Prompts,
    Options,
    Settings,
    Tools,
}

impl HomePaneType {
    pub fn new(&self, app: &mut Application) -> HomePaneTypeWithId {
        app.view_data.counter += 1;
        let count = app.view_data.counter;

        match self {
            Self::Models => {
                app.view_data
                    .home
                    .models
                    .insert(count, ModelsView::default());
                HomePaneTypeWithId::Models(count)
            }
            Self::Prompts => {
                app.view_data
                    .home
                    .prompts
                    .insert(count, PromptsView::default());
                HomePaneTypeWithId::Prompts(count)
            }
            Self::Settings => {
                let mut settings = SettingsView::default();
                settings.instance_url = app.cache.client_settings.instance_url.clone();
                app.view_data.home.settings.insert(count, settings);
                HomePaneTypeWithId::Settings(count)
            }
            Self::Options => {
                app.view_data
                    .home
                    .options
                    .insert(count, OptionsView::default());
                HomePaneTypeWithId::Options(count)
            }
            Self::Pulls => {
                app.view_data.home.pulls.insert(count, PullsView::default());
                HomePaneTypeWithId::Pulls(count)
            }
            _ => HomePaneTypeWithId::Loading,
        }
    }
}

#[derive(Debug, Clone)]
pub enum HomePaneTypeWithId {
    Chat(u32),
    Pulls(u32),
    Models(u32),
    Prompts(u32),
    Options(u32),
    Settings(u32),
    Tools(u32),
    Loading,
}

impl Into<HomePaneType> for &HomePaneTypeWithId {
    fn into(self) -> HomePaneType {
        match self {
            HomePaneTypeWithId::Loading => HomePaneType::Loading,
            HomePaneTypeWithId::Chat(_) => HomePaneType::Chat,
            HomePaneTypeWithId::Pulls(_) => HomePaneType::Pulls,
            HomePaneTypeWithId::Models(_) => HomePaneType::Models,
            HomePaneTypeWithId::Prompts(_) => HomePaneType::Prompts,
            HomePaneTypeWithId::Options(_) => HomePaneType::Options,
            HomePaneTypeWithId::Settings(_) => HomePaneType::Settings,
            HomePaneTypeWithId::Tools(_) => HomePaneType::Tools,
        }
    }
}

impl Into<HomePaneType> for HomePaneTypeWithId {
    fn into(self) -> HomePaneType {
        (&self).into()
    }
}

#[derive(Debug, Clone)]
pub struct HomePanes {
    pub focus: Option<pane_grid::Pane>,
    pub panes: pane_grid::State<HomePaneTypeWithId>,
    pub pick: Option<HomePickingType>,
}

impl HomePanes {
    pub fn new(pane: HomePaneTypeWithId) -> Self {
        let (panes, _) = pane_grid::State::new(pane);
        let (focus, _) = panes.panes.iter().last().unwrap();

        Self {
            focus: Some(focus.clone()),
            panes,
            pick: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PaneMessage {
    Close(pane_grid::Pane),
    NewWindow(pane_grid::Pane),
    Clicked(pane_grid::Pane),
    Dragged(pane_grid::DragEvent),
    Resized(pane_grid::ResizeEvent),
    Split(pane_grid::Axis, pane_grid::Pane, HomePaneType),
    Replace(pane_grid::Pane, HomePaneType),
    ReplaceChat(pane_grid::Pane, String),
    ChatLoaded(pane_grid::Pane, Chat, Vec<MessageMk>),
    Pick(HomePickingType),
    UnPick,
}

impl PaneMessage {
    pub fn handle_new_chat(
        app: &mut Application,
        id: window::Id,
        pane: pane_grid::Pane,
    ) -> Task<Message> {
        if let Some(x) = app.cache.previews.first().map(|x| x.id.key().to_string()) {
            Task::done(Message::Window(WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::ReplaceChat(pane, x))),
            )))
        } else {
            Task::future(async move {
                let req = DATA.read().unwrap().to_request();

                if let Ok(chats) = req
                    .make_request::<Vec<Chat>, ()>("chat/all/", &(), RequestType::Get)
                    .await
                {
                    if chats.len() > 0 {
                        return Message::Batch(vec![
                            match req
                                .make_request::<Preview, ()>(
                                    &format!(
                                        "preview/{}",
                                        chats.first().unwrap().id.key().to_string()
                                    ),
                                    &(),
                                    RequestType::Put,
                                )
                                .await
                                .map(|x| {
                                    Message::Cache(CacheMessage::AddPreview(PreviewMk::from(x)))
                                }) {
                                Ok(x) => x,
                                Err(e) => Message::Err(e),
                            },
                            Message::Window(WindowMessage::Page(
                                id,
                                PageMessage::Home(HomeMessage::Pane(PaneMessage::ReplaceChat(
                                    pane,
                                    chats.first().unwrap().id.key().to_string(),
                                ))),
                            )),
                        ]);
                    }
                }

                match req
                    .make_request::<Chat, ChatData>(
                        "chat/",
                        &ChatData::default(),
                        RequestType::Post,
                    )
                    .await
                {
                    Ok(chat) => Message::Batch(vec![
                        match req
                            .make_request::<Preview, ()>(
                                &format!("preview/{}", chat.id.key().to_string()),
                                &(),
                                RequestType::Put,
                            )
                            .await
                        {
                            Ok(x) => Message::Cache(CacheMessage::AddPreview(PreviewMk::from(x))),
                            Err(e) => Message::Err(e),
                        },
                        Message::Window(WindowMessage::Page(
                            id,
                            PageMessage::Home(HomeMessage::Pane(PaneMessage::ReplaceChat(
                                pane,
                                chat.id.key().to_string(),
                            ))),
                        )),
                    ]),
                    Err(e) => Message::Err(e),
                }
            })
        }
    }
    pub fn handle(self, app: &mut Application, id: window::Id) -> Task<Message> {
        match self {
            Self::Pick(x) => {
                match x {
                    HomePickingType::OpenPane(x) if !app.cache.client_settings.use_panes => {
                        return Task::done(Message::Window(WindowMessage::Page(
                            id,
                            crate::pages::PageMessage::Home(super::message::HomeMessage::Pane(
                                PaneMessage::Replace(
                                    app.get_home_page(&id)
                                        .unwrap()
                                        .panes
                                        .panes
                                        .panes
                                        .first_key_value()
                                        .unwrap()
                                        .0
                                        .clone(),
                                    x,
                                ),
                            )),
                        )));
                    }
                    HomePickingType::ReplaceChat(x) => {
                        let view = app.get_home_page(&id).unwrap();
                        if view.panes.panes.panes.len() > 1 {
                            view.panes.pick = Some(HomePickingType::ReplaceChat(x))
                        } else {
                            return Task::done(Message::Window(WindowMessage::Page(
                                id,
                                PageMessage::Home(HomeMessage::Pane(PaneMessage::ReplaceChat(
                                    view.panes.panes.panes.first_key_value().unwrap().0.clone(),
                                    x,
                                ))),
                            )));
                        }
                    }
                    _ => app.get_home_page(&id).unwrap().panes.pick = Some(x),
                }

                Task::none()
            }
            Self::UnPick => {
                app.get_home_page(&id).unwrap().panes.pick = None;
                Task::none()
            }
            Self::Dragged(pane_grid::DragEvent::Dropped { pane, target }) => {
                app.get_home_page(&id)
                    .unwrap()
                    .panes
                    .panes
                    .drop(pane, target);
                Task::none()
            }
            Self::Dragged(_) => Task::none(),
            Self::Resized(pane_grid::ResizeEvent { split, ratio }) => {
                app.get_home_page(&id)
                    .unwrap()
                    .panes
                    .panes
                    .resize(split, ratio);
                Task::none()
            }
            Self::Clicked(pane) => {
                app.get_home_page(&id).unwrap().panes.focus = Some(pane);
                Task::none()
            }
            Self::Close(pane) => {
                let page = app.get_home_page(&id).unwrap();
                if page.panes.panes.len() <= 1 {
                    return Task::none();
                }

                // TODO remove pane from view_data
                if let Some((_, sibling)) = page.panes.panes.close(pane) {
                    page.panes.focus = Some(sibling);
                }

                Task::none()
            }
            Self::Replace(pane, pane_type) => {
                let value = pane_type.new(app);
                let page = app.get_home_page(&id).unwrap();

                // TODO remove pane from view_data
                let _ = page.panes.panes.panes.insert(pane.clone(), value);

                page.panes.pick = None;
                page.panes.focus = Some(pane);

                if pane_type == HomePaneType::Chat {
                    Self::handle_new_chat(app, id, pane)
                } else {
                    Task::none()
                }
            }
            Self::NewWindow(pane) => {
                let page = app.get_home_page(&id).unwrap();

                if page.panes.panes.len() <= 1 {
                    return Task::none();
                }

                if let Some((value, sibling)) = page.panes.panes.close(pane) {
                    page.panes.focus = Some(sibling);

                    let mut page = HomePage::new();
                    page.panes = HomePanes::new(value);
                    app.view_data.page_stack.push(Pages::Home(page));
                } else {
                    return Task::none();
                }

                Task::done(Message::Window(WindowMessage::OpenWindow))
            }

            Self::Split(axis, pane, pane_type) => {
                let value = pane_type.new(app);
                let page = app.get_home_page(&id).unwrap();
                let result = page.panes.panes.split(axis, pane, value);

                let pane = if let Some((p, _)) = result {
                    page.panes.focus = Some(p.clone());
                    p
                } else {
                    page.panes.pick = None;
                    return Task::none();
                };

                page.panes.pick = None;

                if pane_type == HomePaneType::Chat {
                    Self::handle_new_chat(app, id, pane)
                } else {
                    Task::none()
                }
            }
            PaneMessage::ReplaceChat(pane, chat_id) => Task::future(async move {
                let req = DATA.read().unwrap().to_request();

                let chat: Chat = match req
                    .make_request(&format!("chat/{}", chat_id), &(), RequestType::Get)
                    .await
                {
                    Ok(x) => x,
                    Err(e) => return Message::Err(e),
                };

                let msgs = if let Some(x) = chat.root.clone() {
                    match MessagesData::load_chat_from_root(x, None).await {
                        Ok(x) => x,
                        Err(e) => return Message::Err(e),
                    }
                } else {
                    Vec::new()
                };

                Message::Window(WindowMessage::Page(
                    id,
                    PageMessage::Home(HomeMessage::Pane(PaneMessage::ChatLoaded(pane, chat, msgs))),
                ))
            }),
            PaneMessage::ChatLoaded(pane, chat, messages) => {
                let messages = app.cache.home_shared.messages.push(messages);

                app.view_data.counter += 1;
                let count = app.view_data.counter;

                app.view_data.home.chats.insert(
                    count,
                    ChatsView {
                        input: text_editor::Content::default(),
                        models: if let Some(model) =
                            app.cache.client_settings.default_provider.clone()
                        {
                            vec![model]
                        } else if let Some(x) =
                            DATA.read().unwrap().models.first().map(|x| x.clone())
                        {
                            vec![x]
                        } else {
                            Vec::new()
                        },
                        path: Vec::new(),
                        start: 0,
                        messages,
                        chat,
                        edits: HashMap::new(),
                        expanded_messages: Vec::new(),
                        prompts: PromptsData::default(),
                        selected_prompt: None,
                    },
                );

                let value = HomePaneTypeWithId::Chat(count);
                let page = app.get_home_page(&id).unwrap();

                // TODO remove pane from view_data
                let _ = page.panes.panes.panes.insert(pane.clone(), value);

                page.panes.pick = None;
                page.panes.focus = Some(pane);
                Task::none()
            }
        }
    }
}
