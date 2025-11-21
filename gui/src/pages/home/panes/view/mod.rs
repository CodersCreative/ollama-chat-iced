use std::fmt::Display;

use iced::{
    Element, Padding,
    alignment::{Horizontal, Vertical},
    widget::{center, column, container, horizontal_rule, horizontal_space, pane_grid, row, text},
    window,
};

use crate::{
    Application, Message,
    font::{BODY_SIZE, HEADER_SIZE},
    pages::{
        PageMessage,
        home::{
            message::{HomeMessage, HomePickingType},
            panes::{HomePaneTypeWithId, HomePanes, PaneMessage},
        },
    },
    style,
    windows::message::WindowMessage,
};

pub mod downloads;
pub mod models;
pub mod options;
pub mod settings;

fn add_to_window<'a>(
    app: &'a Application,
    id: window::Id,
    pane: pane_grid::Pane,
    title: String,
    action: Option<HomePickingType>,
    child: Element<'a, Message>,
) -> Element<'a, Message> {
    match action {
        Some(HomePickingType::ReplaceChat(x)) => {}

        Some(HomePickingType::OpenPane(pick)) => {
            return container(center(row![
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
            ]))
            .style(style::container::window_back_danger)
            .into();
        }
        None => {}
    };

    let header = container(
        row![
            text(title)
                .color(app.theme().palette().primary)
                .size(BODY_SIZE)
                .align_y(Vertical::Center)
                .align_x(Horizontal::Left),
            horizontal_space(),
            style::svg_button::danger("close.svg", BODY_SIZE).on_press(Message::Window(
                WindowMessage::Page(
                    id,
                    PageMessage::Home(HomeMessage::Pane(PaneMessage::Close(pane.clone())))
                )
            ))
        ]
        .align_y(Vertical::Center),
    );

    container(
        column![
            header,
            horizontal_rule(1).style(style::rule::translucent::text),
            child,
        ]
        .spacing(5),
    )
    .style(style::container::window_back)
    .padding(Padding::default().top(5).bottom(5).left(5).right(5))
    .into()
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

                pane_grid::Content::new(add_to_window(
                    app,
                    id.clone(),
                    pane,
                    state.to_string(),
                    action,
                    state.view(app, id.clone()),
                ))
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
            }),
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
                Self::Downloads(_) => "Downloads",
                Self::Models(_) => "Ollama Models",
                Self::Prompts(_) => "Prompts",
                Self::Options(_) => "Generation Options",
                Self::Settings(_) => "Settings",
                Self::Tools(_) => "Tools",
            }
        )
    }
}

impl HomePaneTypeWithId {
    pub fn view<'a>(&'a self, app: &'a Application, id: window::Id) -> Element<'a, Message> {
        match self {
            _ => text("Hello, World!!!").into(),
        }
    }
}
