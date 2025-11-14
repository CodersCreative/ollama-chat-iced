pub mod data;
pub mod pages;
pub mod style;
pub mod utils;
pub mod windows;

use iced::{Element, Font, Subscription, Task, Theme, widget::text, window};
use std::collections::BTreeMap;

use crate::{
    pages::Pages,
    windows::{Window, message::WindowMessage},
};

pub mod font {
    pub const FONT: &[u8] = include_bytes!("../assets/RobotoMonoNerdFont-Regular.ttf");
    pub const HEADER_SIZE: u16 = 24;
    pub const SUB_HEADING_SIZE: u16 = 16;
    pub const BODY_SIZE: u16 = 12;
    pub const SMALL_SIZE: u16 = 8;
}

#[derive(Debug, Clone)]
pub struct Application {
    pub data: data::Data,
    pub windows: BTreeMap<window::Id, Window>,
}

fn main() -> iced::Result {
    let font = Font {
        family: iced::font::Family::Name("Roboto"),
        style: iced::font::Style::Normal,
        stretch: iced::font::Stretch::Normal,
        weight: iced::font::Weight::Normal,
    };
    iced::daemon(Application::title, Application::update, Application::view)
        .subscription(Application::subscription)
        .font(font::FONT)
        .default_font(font)
        .theme(Application::theme)
        .run_with(Application::new)
}

#[derive(Debug, Clone)]
pub enum Message {
    None,
    Window(WindowMessage),
    SetData(Result<data::Data, String>),
}

impl Application {
    pub fn new() -> (Self, Task<Message>) {
        let (_, open) = window::open(window::Settings::default());
        (
            Self {
                data: data::Data::default(),
                windows: BTreeMap::new(),
            },
            Task::batch([
                open.map(|id| Message::Window(WindowMessage::WindowOpened(id, Pages::default()))),
                Task::future(async move {
                    Message::SetData(data::Data::get(None).await.map_err(|e| e.to_string()))
                }),
            ]),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::None => Task::none(),
            Message::Window(message) => message.handle(self),
            Message::SetData(d) => {
                match d {
                    Ok(d) => self.data = d,
                    Err(e) => println!("{e}"),
                }

                Task::none()
            }
        }
    }

    pub fn title(&self, _window: window::Id) -> String {
        String::from("OChat")
    }

    pub fn view<'a>(&'a self, window_id: window::Id) -> Element<'a, Message> {
        if let Some(window) = self.windows.get(&window_id) {
            window.view(self, &window_id)
        } else {
            text("Window Not Found").into()
        }
    }

    pub fn theme(&self, _window: window::Id) -> Theme {
        Theme::CatppuccinMocha
    }

    pub fn subscription(&self) -> Subscription<Message> {
        window::close_events().map(|id| Message::Window(WindowMessage::WindowClosed(id)))
    }
}
