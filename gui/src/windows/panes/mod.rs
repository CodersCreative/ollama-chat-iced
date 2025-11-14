pub mod message;
pub mod view;

use crate::{
    ChatApp, Message,
    common::Id,
    models::view::Models,
    options::view::Options,
    prompts::view::Prompts,
    settings::view::Settings,
    style::{self},
    tools::view::Tools,
    utils::get_path_assets,
    windows::panes::message::PaneMessage,
};
use iced::{Element, Length};
use iced::{
    Padding, Renderer, Theme,
    alignment::{Horizontal, Vertical},
    widget::{
        button, center, column, container, horizontal_space, mouse_area, pane_grid, row, svg, text,
    },
};

#[derive(Debug, Clone)]
pub enum Pane {
    Options(Id),
    Settings(Id),
    Tools(Id),
    Chat(Id),
    Models(Id),
    Prompts(Id),
    #[cfg(feature = "voice")]
    Call,
    NoModel,
}

impl Pane {
    pub fn new_options(app: &mut ChatApp, model: String) -> Self {
        let option = (Id::new(), Options::new(model.clone()));
        app.main_view.add_to_options(option.0.clone(), option.1);
        return Self::Options(option.0);
    }

    pub fn new_settings(app: &mut ChatApp) -> Self {
        let settings = (Id::new(), Settings::new(app));
        app.main_view
            .add_to_settings(settings.0.clone(), settings.1);
        return Self::Settings(settings.0);
    }

    pub fn new_tools(app: &mut ChatApp) -> Self {
        let tools = (Id::new(), Tools::new(app));
        app.main_view.add_to_tools(tools.0.clone(), tools.1);
        return Self::Tools(tools.0);
    }

    pub fn new_models(app: &mut ChatApp) -> Self {
        let model = (Id::new(), Models::new(app));
        app.main_view.add_model(model.0.clone(), model.1);
        return Self::Models(model.0);
    }

    pub fn new_prompts(app: &mut ChatApp) -> Self {
        let prompt = (Id::new(), Prompts::new(app));
        app.main_view.add_prompt(prompt.0.clone(), prompt.1);
        return Self::Prompts(prompt.0);
    }
}

#[derive(Debug, Clone)]
pub struct Panes {
    pub focus: Option<pane_grid::Pane>,
    pub panes: pane_grid::State<Pane>,
    pub pick: Option<(pane_grid::Pane, Pane)>,
    #[cfg(feature = "voice")]
    pub call: Option<pane_grid::Pane>,
    pub last_chat: Id,
    pub created: usize,
}

impl Panes {
    pub fn new(pane: Pane) -> Self {
        let chat = if let Pane::Chat(x) = &pane {
            x.clone()
        } else {
            Id::new()
        };
        let (panes, _) = pane_grid::State::new(pane);
        let (focus, _) = panes.panes.iter().last().unwrap();

        Self {
            focus: Some(focus.clone()),
            panes,
            #[cfg(feature = "voice")]
            call: None,
            pick: None,
            created: 1,
            last_chat: chat,
        }
    }
}

fn window_button<'a>(title: &'a str, size: u16) -> button::Button<'a, Message, Theme, Renderer> {
    button(
        svg(svg::Handle::from_path(get_path_assets(title.to_string())))
            .style(style::svg::white)
            .width(Length::Fixed(size as f32)),
    )
    .style(style::button::transparent_text)
}

pub fn add_to_window<'a>(
    app: &'a ChatApp,
    pane: pane_grid::Pane,
    state: Pane,
    title: &'a str,
    picking: Option<Pane>,
    child: Element<'a, Message>,
) -> Element<'a, Message> {
    if let Some(pick) = picking {
        return container(center(row![
            window_button("vertical.svg", 48).on_press(Message::Pane(PaneMessage::Split(
                pane_grid::Axis::Vertical,
                pane,
                pick.clone()
            ))),
            window_button("restart.svg", 48)
                .on_press(Message::Pane(PaneMessage::Replace(pane, pick.clone()))),
            window_button("close.svg", 48).on_press(Message::Pane(PaneMessage::UnPick)),
            window_button("horizontal.svg", 48).on_press(Message::Pane(PaneMessage::Split(
                pane_grid::Axis::Horizontal,
                pane,
                pick.clone()
            ))),
        ]))
        .style(style::container::window_back)
        .into();
    }

    let header = container(
        row![
            text(title)
                .color(app.theme().palette().primary)
                .size(16)
                .align_y(Vertical::Center)
                .align_x(Horizontal::Left),
            horizontal_space(),
            window_button("add_chat.svg", 16).on_press(Message::Pane(PaneMessage::Pick(
                pane,
                Pane::Chat(app.panes.last_chat)
            ))),
            window_button("star.svg", 16).on_press(Message::Pane(PaneMessage::Pick(
                pane,
                Pane::Models(Id::new())
            ))),
            window_button("prompt.svg", 16).on_press(Message::Pane(PaneMessage::Pick(
                pane,
                Pane::Prompts(Id::new())
            ))),
            window_button("tools.svg", 16).on_press(Message::Pane(PaneMessage::Pick(
                pane,
                Pane::Tools(Id::new())
            ))),
            window_button("ai.svg", 16).on_press(Message::Pane(PaneMessage::Pick(
                pane,
                Pane::Options(Id::new())
            ))),
            window_button("settings.svg", 16).on_press(Message::Pane(PaneMessage::Pick(
                pane,
                Pane::Settings(Id::new())
            ))),
            window_button("close.svg", 16).on_press(Message::Pane(PaneMessage::Close(pane)))
        ]
        .align_y(Vertical::Center),
    )
    .padding(Padding::default().top(5).bottom(5).left(30).right(30));
    mouse_area(column![header, child,])
        .on_press(Message::Pane(PaneMessage::Clicked(pane, state)))
        .into()
}

impl Panes {
    pub fn new_pane(app: &mut ChatApp, grid_pane: pane_grid::Pane, pane: Pane) {
        let value = match pane {
            Pane::Options(_) => {
                if let Some(model) = app.logic.models.first() {
                    Pane::new_options(app, model.to_string())
                } else {
                    return;
                }
            }
            Pane::Tools(_) => Pane::new_tools(app),
            Pane::Settings(_) => Pane::new_settings(app),
            Pane::Chat(x) => {
                if let Some(chat) = app.main_view.chats().get(&x) {
                    let id = Id::new();
                    app.main_view.add_to_chats(id.clone(), chat.clone());
                    Pane::Chat(id)
                } else {
                    return;
                }
            }
            Pane::Models(_) => Pane::new_models(app),
            Pane::Prompts(_) => Pane::new_prompts(app),
            #[cfg(feature = "voice")]
            Pane::Call => Pane::Call,
            _ => Pane::NoModel,
        };

        #[cfg(feature = "voice")]
        if let Pane::Call = pane {
            if let Some(_) = app.panes.call {
                return;
            }
        }

        if app.save.use_panes {
            app.panes.pick = Some((grid_pane.clone(), value));
        } else {
            let result = app
                .panes
                .panes
                .split(pane_grid::Axis::Vertical, grid_pane, value.clone());

            if let Some((p, _)) = result {
                app.panes.focus = Some(p);
                #[cfg(feature = "voice")]
                if let Pane::Call = pane {
                    app.panes.call = Some(p);
                }
            }

            app.panes.pick = None;

            if let Pane::Chat(x) = pane {
                app.panes.last_chat = x;
            }
            app.panes.panes.close(grid_pane);
        }
    }
}
