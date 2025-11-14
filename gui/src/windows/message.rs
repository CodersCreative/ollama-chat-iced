use iced::{Task, Vector, widget::text_input, window};

use crate::{Application, Message, pages::Pages, windows::Window};

#[derive(Debug, Clone)]
pub enum WindowMessage {
    OpenWindow,
    WindowOpened(window::Id, Pages),
    WindowClosed(window::Id),
}

impl WindowMessage {
    pub fn handle(self, app: &mut Application) -> Task<Message> {
        match self {
            Self::OpenWindow => {
                let Some(last_window) = app.windows.keys().last() else {
                    return Task::none();
                };

                window::get_position(*last_window)
                    .then(|last_position| {
                        let position =
                            last_position.map_or(window::Position::Default, |last_position| {
                                window::Position::Specific(last_position + Vector::new(20.0, 20.0))
                            });

                        let (_id, open) = window::open(window::Settings {
                            position,
                            ..window::Settings::default()
                        });

                        open
                    })
                    .map(|id| Message::Window(WindowMessage::WindowOpened(id, Pages::default())))
            }
            Self::WindowClosed(id) => {
                app.windows.remove(&id);

                if app.windows.is_empty() {
                    iced::exit()
                } else {
                    Task::none()
                }
            }
            Self::WindowOpened(id, page) => {
                let window = Window::new(page);
                let focus_input = text_input::focus(format!("input-{id}"));

                app.windows.insert(id, window);

                focus_input
            }
        }
    }
}
