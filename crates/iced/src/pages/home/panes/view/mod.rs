use crate::{
    Application, Message,
    font::{BODY_SIZE, HEADER_SIZE, get_bold_font},
    pages::{
        PageMessage,
        home::{
            message::{HomeMessage, HomePickingType},
            panes::{
                HomePaneTypeWithId, HomePanes, PaneMessage,
                view::{
                    call::{CallView, CallViewMessage},
                    chat::{ChatsView, ChatsViewMessage},
                    models::{ModelsView, ModelsViewMessage},
                    options::{OptionsView, OptionsViewMessage},
                    prompts::{PromptsView, PromptsViewMessage},
                    pulls::{PullsView, PullsViewMessage},
                    settings::{SettingsView, SettingsViewMessage},
                },
            },
        },
        info,
    },
    style,
    utils::get_path_assets,
    windows::message::WindowMessage,
};
use iced::{
    Element, Padding, Task,
    alignment::{Horizontal, Vertical},
    widget::{center, column, container, pane_grid, row, svg},
    window,
};
use iced_selection::text;
use std::{collections::HashMap, fmt::Display};

pub mod call;
pub mod chat;
pub mod models;
pub mod options;
pub mod prompts;
pub mod pulls;
pub mod settings;

#[derive(Debug, Clone, Default)]
pub struct HomePaneViewData {
    pub models: HashMap<u32, ModelsView>,
    pub settings: HashMap<u32, SettingsView>,
    pub prompts: HashMap<u32, PromptsView>,
    pub options: HashMap<u32, OptionsView>,
    pub pulls: HashMap<u32, PullsView>,
    pub chats: HashMap<u32, ChatsView>,
    pub call: Option<CallView>,
}

#[derive(Debug, Clone)]
pub enum HomePaneViewMessage {
    Models(u32, ModelsViewMessage),
    Prompts(u32, PromptsViewMessage),
    Options(u32, OptionsViewMessage),
    Pulls(u32, PullsViewMessage),
    Settings(u32, SettingsViewMessage),
    Chats(u32, ChatsViewMessage),
    Call(CallViewMessage),
}

impl HomePaneViewMessage {
    pub fn handle(self, app: &mut Application) -> Task<Message> {
        match self {
            Self::Models(id, x) => x.handle(app, id),
            Self::Prompts(id, x) => x.handle(app, id),
            Self::Options(id, x) => x.handle(app, id),
            Self::Pulls(id, x) => x.handle(app, id),
            Self::Settings(id, x) => x.handle(app, id),
            Self::Chats(id, x) => x.handle(app, id),
            Self::Call(x) => x.handle(app),
        }
    }
}

fn add_to_window<'a>(
    id: window::Id,
    pane: pane_grid::Pane,
    title: String,
    action: Option<HomePickingType>,
    child: Element<'a, Message>,
) -> pane_grid::Content<'a, Message> {
    let header = pane_grid::TitleBar::new(
        text(title)
            .font(get_bold_font())
            .style(style::text::primary)
            .size(BODY_SIZE + 2)
            .align_y(Vertical::Center)
            .align_x(Horizontal::Left),
    )
    .controls(pane_grid::Controls::new(row![
        style::svg_button::text("add_window.svg", BODY_SIZE + 2).on_press(Message::Window(
            WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::NewWindow(pane.clone()))),
            ),
        )),
        style::svg_button::danger("close.svg", BODY_SIZE + 2).on_press(Message::Window(
            WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::Close(pane.clone()))),
            ),
        )),
    ]))
    .style(style::container::window_title_back)
    .padding(5);

    pane_grid::Content::new(
        container(match action.clone() {
            Some(HomePickingType::ReplaceChat(x)) => container(center(row![
                style::svg_button::text("restart.svg", HEADER_SIZE * 2).on_press(Message::Window(
                    WindowMessage::Page(
                        id,
                        PageMessage::Home(HomeMessage::Pane(PaneMessage::ReplaceChat(
                            pane,
                            x.clone()
                        )))
                    )
                )),
                style::svg_button::text("close.svg", HEADER_SIZE * 2).on_press(Message::Window(
                    WindowMessage::Page(
                        id,
                        PageMessage::Home(HomeMessage::Pane(PaneMessage::UnPick))
                    )
                )),
            ])),
            Some(HomePickingType::OpenPane(pick)) => container(center(row![
                style::svg_button::text("vertical.svg", HEADER_SIZE * 2).on_press(Message::Window(
                    WindowMessage::Page(
                        id,
                        PageMessage::Home(HomeMessage::Pane(PaneMessage::Split(
                            pane_grid::Axis::Vertical,
                            pane,
                            pick.clone()
                        )))
                    )
                )),
                style::svg_button::text("restart.svg", HEADER_SIZE * 2).on_press(Message::Window(
                    WindowMessage::Page(
                        id,
                        PageMessage::Home(HomeMessage::Pane(PaneMessage::Replace(
                            pane,
                            pick.clone()
                        )))
                    )
                )),
                style::svg_button::text("close.svg", HEADER_SIZE * 2).on_press(Message::Window(
                    WindowMessage::Page(
                        id,
                        PageMessage::Home(HomeMessage::Pane(PaneMessage::UnPick))
                    )
                )),
                style::svg_button::text("horizontal.svg", HEADER_SIZE * 2).on_press(
                    Message::Window(WindowMessage::Page(
                        id,
                        PageMessage::Home(HomeMessage::Pane(PaneMessage::Split(
                            pane_grid::Axis::Horizontal,
                            pane,
                            pick.clone()
                        )))
                    ))
                )
            ])),
            _ => container(child),
        })
        .style(match action {
            None => style::container::window_back,
            Some(_) => style::container::window_back_danger,
        })
        .padding(5),
    )
    .title_bar(header)
}

impl HomePanes {
    pub fn view<'a>(&'a self, app: &'a Application, id: window::Id) -> Element<'a, Message> {
        container(
            pane_grid(&self.panes, |pane, state, _is_maximised| {
                let action = match (&self.pick, state) {
                    (Some(HomePickingType::OpenPane(x)), _) => {
                        Some(HomePickingType::OpenPane(x.clone()))
                    }
                    (Some(HomePickingType::ReplaceChat(x)), HomePaneTypeWithId::Chat(_)) => {
                        Some(HomePickingType::ReplaceChat(x.clone()))
                    }
                    _ => None,
                };

                add_to_window(
                    id.clone(),
                    pane,
                    state.to_string(),
                    action,
                    state.view(app, id.clone()),
                )
            })
            .on_click(move |x| {
                Message::Window(WindowMessage::Page(
                    id,
                    PageMessage::Home(HomeMessage::Pane(PaneMessage::Clicked(x))),
                ))
            })
            .on_drag(move |x| {
                Message::Window(WindowMessage::Page(
                    id,
                    PageMessage::Home(HomeMessage::Pane(PaneMessage::Dragged(x))),
                ))
            })
            .on_resize(10, move |x| {
                Message::Window(WindowMessage::Page(
                    id,
                    PageMessage::Home(HomeMessage::Pane(PaneMessage::Resized(x))),
                ))
            })
            .spacing(10),
        )
        .padding(10)
        .into()
    }
}

impl Display for HomePaneTypeWithId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Chat(_) => "Chat",
                Self::Pulls(_) => "Pulls",
                Self::Models(_) => "Ollama Models",
                Self::Prompts(_) => "Prompts",
                Self::Options(_) => "Generation Options",
                Self::Settings(_) => "Settings",
                Self::Tools(_) => "Tools",
                Self::NotImplemented => "Coming Soon...",
                Self::Call => "Call",
                Self::Loading => "Loading",
                Self::Info => "Info",
            }
        )
    }
}

impl HomePaneTypeWithId {
    pub fn view<'a>(&'a self, app: &'a Application, _id: window::Id) -> Element<'a, Message> {
        match self {
            Self::Models(x) => app.view_data.home.models.get(x).unwrap().view(app, *x),
            Self::Prompts(x) => app.view_data.home.prompts.get(x).unwrap().view(app, *x),
            Self::Options(x) => app.view_data.home.options.get(x).unwrap().view(app, *x),
            Self::Pulls(x) => app.view_data.home.pulls.get(x).unwrap().view(app, *x),
            Self::Settings(x) => app.view_data.home.settings.get(x).unwrap().view(app, *x),
            Self::Chat(x) => app.view_data.home.chats.get(x).unwrap().view(app, *x),
            Self::Call => app.view_data.home.call.as_ref().unwrap().view(app),
            Self::Loading => center(
                container(column![
                    text("Loading...")
                        .style(style::text::primary)
                        .size(HEADER_SIZE),
                    text("If its taking an extended period of time ( > 1 minute) then:\ntry clicking on 'New Chat' or on a pre-existing chat")
                        .style(style::text::translucent::text)
                        .size(BODY_SIZE),
                ])
                .max_width(800)
                .padding(Padding::new(20.0))
                .style(style::container::neutral_back),
            )
            .into(),
            Self::NotImplemented => center(
                container(column![
                    text("Coming soon...")
                        .style(style::text::primary)
                        .size(HEADER_SIZE),
                    svg(svg::Handle::from_path(get_path_assets("tools.svg"))).width(400).height(400).style(style::svg::text),
                    text("Unfortunately this page is still under construction and will be coming in a later update!")
                        .style(style::text::translucent::text)
                        .size(BODY_SIZE),
                ].align_x(Horizontal::Center))
                .max_width(800)
                .padding(Padding::new(20.0))
                .style(style::container::neutral_back),
            )
            .into(),
            Self::Info => info::view(app),
            _ => text("Hello, World!!!").into(),
        }
    }
}
