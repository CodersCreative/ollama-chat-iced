pub mod data;
pub mod pages;
pub mod style;
pub mod utils;
pub mod windows;

use iced::{
    Element, Font, Subscription, Task, Theme,
    widget::{markdown, text},
    window,
};
use ochat_types::chats::previews::Preview;
use std::{
    collections::BTreeMap,
    sync::{LazyLock, RwLock},
};

use crate::{
    pages::{Pages, home::sidebar::PreviewMk},
    windows::{Window, message::WindowMessage},
};

pub mod font {
    pub const FONT: &[u8] = include_bytes!("../assets/RobotoMonoNerdFont-Regular.ttf");
    pub const HEADER_SIZE: u16 = 24;
    pub const SUB_HEADING_SIZE: u16 = 16;
    pub const BODY_SIZE: u16 = 12;
    pub const SMALL_SIZE: u16 = 8;
}

static DATA: LazyLock<RwLock<data::Data>> = LazyLock::new(|| {
    RwLock::new(
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(data::Data::get(None))
            .unwrap(),
    )
});

#[derive(Debug, Clone)]
pub struct Application {
    pub windows: BTreeMap<window::Id, Window>,
    pub previews: Vec<PreviewMk>,
    pub theme: Theme,
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
    SetPreviews(Vec<Preview>),
}

impl Application {
    pub fn new() -> (Self, Task<Message>) {
        drop(DATA.read());
        let (_, open) = window::open(window::Settings::default());
        (
            Self {
                windows: BTreeMap::new(),
                theme: Theme::CatppuccinMocha,
                previews: Vec::new(),
            },
            open.map(|id| Message::Window(WindowMessage::WindowOpened(id, Pages::default())))
                .chain(Task::future(async {
                    let req = DATA.read().unwrap().to_request();

                    let previews = req
                        .make_request("preview/all/", &(), data::RequestType::Get)
                        .await
                        .unwrap_or_default();

                    Message::SetPreviews(previews)
                })),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::None => Task::none(),
            Message::Window(message) => message.handle(self),
            Message::SetPreviews(previews) => {
                self.previews = previews.into_iter().map(|x| x.into()).collect();
                Task::none()
            }
        }
    }

    pub fn title(&self, _window: window::Id) -> String {
        String::from("OChat")
    }

    pub fn view<'a>(&'a self, window_id: window::Id) -> Element<'a, Message> {
        if let Some(window) = self.windows.get(&window_id) {
            window.view(self, window_id)
        } else {
            text("Window Not Found").into()
        }
    }

    pub fn theme(&self, _window: window::Id) -> Theme {
        self.theme.clone()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        window::close_events().map(|id| Message::Window(WindowMessage::WindowClosed(id)))
    }
}
