use iced::{alignment::{Horizontal, Vertical}, widget::{button, center, column, container, horizontal_space, image, markdown, pane_grid, row, scrollable::{self, Direction, Scrollbar}, text}, Padding, Pixels, Renderer, Task, Theme};
use iced::{Element, Length};
use ollama_rs::generation::chat::ChatMessage;
use serde::{Deserialize, Serialize};
use crate::{options::Options, save::chats::Chats, style::{self}, utils::{convert_image, generate_id}, ChatApp, Message};

#[derive(Debug, Clone)]
pub enum Pane{
    Settings(i32),
    Chat(i32),
    NoModel,
}

impl Pane {
    pub fn new_settings(app : &mut ChatApp, model : String) -> Self{
        let model = Options::new(model.clone());
        app.main_view.options.push(model.clone());
        return Self::Settings(model.1);
    }
}

#[derive(Debug, Clone)]
pub struct Panes{
    pub focus : Option<pane_grid::Pane>,
    pub panes : pane_grid::State<Pane>,
    pub pick : Option<(pane_grid::Pane, Pane)>,
    pub last_chat : i32,
    pub created : usize,
}

impl Panes {
    pub fn new(pane: Pane) -> Self{
        //let (panes, _) = pane_grid::State::new(Pane::Chat(0));
        let (panes, _) = pane_grid::State::new(pane);
        Self{
            focus: None,
            panes,
            pick: None,
            created: 1,
            last_chat: 0,
        }
    }


}

fn window_button<'a>(title : &'a str, size : u16) -> button::Button<'a, Message, Theme, Renderer>{
    button(
        text(title).align_x(Horizontal::Center).align_y(Vertical::Center).size(Pixels::from(size))
    )
    .style(style::button::transparent_text)
    //.on_press(Message::ShowSettings)
}
pub fn add_to_window<'a>(app : &'a ChatApp, pane : pane_grid::Pane, title : &'a str, picking : Option<Pane>, child : Element<'a, Message>) -> Element<'a, Message>{
    if let Some(pick) = picking{
        return container(center(row![
            window_button("|", 48).on_press(Message::Pane(PaneMessage::Split(pane_grid::Axis::Vertical, pane, pick.clone()))),
            window_button("x", 48).on_press(Message::Pane(PaneMessage::UnPick)),
            window_button("-", 48).on_press(Message::Pane(PaneMessage::Split(pane_grid::Axis::Horizontal, pane, pick.clone()))),
        ])).style(style::container::chat_back_ai).into()
    }

    let header = container(row![
        text(title)
        .color(app.theme().palette().primary)
        .size(16)
        .align_y(Vertical::Center)
        .align_x(Horizontal::Left),
        horizontal_space(),
        window_button("+", 16).on_press(Message::Pane(PaneMessage::Pick(pane, Pane::Chat(app.panes.last_chat)))),
        window_button("=", 16).on_press(Message::Pane(PaneMessage::Pick(pane, Pane::Settings(0)))),
        window_button("x", 16).on_press(Message::Pane(PaneMessage::Close(pane)))
    ].align_y(Vertical::Center)).padding(Padding::default().top(5).bottom(5).left(30).right(30));

    column![
        header,
        child,
    ].into()
}

#[derive(Debug, Clone)]
pub enum PaneMessage{
    Clicked(pane_grid::Pane),
    Pick(pane_grid::Pane, Pane),
    UnPick,
    Close(pane_grid::Pane),
    PaneDragged(pane_grid::DragEvent),
    PaneResized(pane_grid::ResizeEvent),
    Split(pane_grid::Axis, pane_grid::Pane, Pane),
}

impl PaneMessage{
    pub fn handle(&self, app : &mut ChatApp) -> Task<Message>{
        match self{
            Self::Clicked(pane) => {
                app.panes.focus = Some(*pane);
                //if let Pane::Chat(x) = pane{
                //
                //}
                Task::none()
            },
            Self::Close(pane) => {
                if app.panes.created > 1{
                    if let Some((_, sibling)) = app.panes.panes.close(*pane) {
                        app.panes.focus = Some(sibling);
                    }
                }
                Task::none()
            }
            Self::PaneDragged(pane_grid::DragEvent::Dropped {
                pane,
                target,
            }) => {
                app.panes.panes.drop(*pane, *target);
                Task::none()
            },
            Self::PaneDragged(_) => {
                Task::none()
            },
            Self::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                app.panes.panes.resize(*split, *ratio);
                Task::none()
            },
            Self::Pick(grid_pane, pane) => {
                app.panes.pick = Some((grid_pane.clone(), match pane {
                    Pane::Settings(_) => Pane::new_settings(app, app.logic.models.first().unwrap().clone()),
                    Pane::Chat(x) => {
                        let mut chat = Chats::get_from_id(app, x.clone()).clone();
                        let id = generate_id();
                        chat.id = id;
                        app.main_view.chats.push(chat);
                        Pane::Chat(id)
                    },
                    _ => Pane::NoModel,
                }));
                Task::none()
            },
            Self::UnPick => {
                app.panes.pick = None;
                Task::none()
            },
            Self::Split(axis, og, pane) => {
                let result = app.panes.panes.split(*axis, *og, pane.clone());

                if let Some((_pane, _)) = result {
                    app.panes.focus = Some(*og);
                }

                app.panes.pick = None;
                if let Pane::Chat(x) = pane{
                    app.panes.last_chat = *x;
                }
                app.panes.created += 1;
                Task::none()
            }
        }
    }
}

impl Panes{
    pub fn view<'a>(&'a self, app : &'a ChatApp) -> Element<'a, Message>{
        pane_grid(&self.panes, |pane, state, is_maximized| {
            let pick = match &app.panes.pick{
                Some(x) => {
                    if pane == x.0{
                        Some(x.1.clone())
                    }else{
                        None
                    }
                },
                None => None
            };

            let options_view = |x : i32| -> Element<Message> {
                Options::get_from_id(app, x).view(app)
            };

            pane_grid::Content::new(match state{
                Pane::Settings(x) => add_to_window(app, pane, "Settings", pick, options_view(*x)),
                Pane::Chat(x) => {
                    let index = Chats::get_index(app, x.clone());
                    //app.main_view.chats.
                    add_to_window(app, pane, "Chat", pick, app.main_view.chats[index.clone()].chat_view(app))
                },
                Pane::NoModel => text("Please install Ollama to use this app.").into(),
            })
        })
        .on_drag(|x| Message::Pane(PaneMessage::PaneDragged(x)))
        .on_resize(10, |x| Message::Pane(PaneMessage::PaneResized(x)))
        .width(Length::FillPortion(50))
        .into()
    }
}
