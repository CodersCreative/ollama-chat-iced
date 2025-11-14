pub mod pages;
pub mod windows;

use iced::{
    Element, Font, Subscription, Task, Theme, Vector,
    widget::{button, text, text_input},
    window,
};
use std::collections::BTreeMap;

use crate::{
    pages::Pages,
    windows::{Window, WindowMessage},
};
pub const FONT: &[u8] = include_bytes!("../assets/RobotoMonoNerdFont-Regular.ttf");

#[derive(Debug, Clone)]
pub struct Application {
    windows: BTreeMap<window::Id, Window>,
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
        .font(FONT)
        .default_font(font)
        .theme(Application::theme)
        .run_with(Application::new)
}

#[derive(Debug, Clone)]
pub enum Message {
    None,
    Window(WindowMessage),
}

impl Application {
    pub fn new() -> (Self, Task<Message>) {
        let (_, open) = window::open(window::Settings::default());
        (
            Self {
                windows: BTreeMap::new(),
            },
            open.map(|id| Message::Window(WindowMessage::WindowOpened(id, Pages::default()))),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::None => Task::none(),
            Message::Window(message) => match message {
                WindowMessage::OpenWindow => {
                    let Some(last_window) = self.windows.keys().last() else {
                        return Task::none();
                    };

                    window::get_position(*last_window)
                        .then(|last_position| {
                            let position =
                                last_position.map_or(window::Position::Default, |last_position| {
                                    window::Position::Specific(
                                        last_position + Vector::new(20.0, 20.0),
                                    )
                                });

                            let (_id, open) = window::open(window::Settings {
                                position,
                                ..window::Settings::default()
                            });

                            open
                        })
                        .map(|id| {
                            Message::Window(WindowMessage::WindowOpened(id, Pages::default()))
                        })
                }
                WindowMessage::WindowClosed(id) => {
                    self.windows.remove(&id);

                    if self.windows.is_empty() {
                        iced::exit()
                    } else {
                        Task::none()
                    }
                }
                WindowMessage::WindowOpened(id, page) => {
                    let window = Window::new(page);
                    let focus_input = text_input::focus(format!("input-{id}"));

                    self.windows.insert(id, window);

                    focus_input
                }
            },
        }
    }

    pub fn title(&self, _window: window::Id) -> String {
        String::from("OChat")
    }

    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if let Some(window) = self.windows.get(&window_id) {
            button("Hello, World!")
                .on_press(Message::Window(WindowMessage::OpenWindow))
                .into()
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
