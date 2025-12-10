use iced::{
    Task, Vector,
    widget::{operation, text_input},
    window,
};

use crate::{Application, Message, pages::PageMessage, windows::Window};

#[derive(Debug, Clone)]
pub enum WindowMessage {
    OpenWindow,
    Page(window::Id, PageMessage),
    WindowOpened(window::Id),
    WindowClosed(window::Id),
}

impl WindowMessage {
    pub fn handle<'a>(self, app: &'a mut Application) -> Task<Message> {
        match self {
            Self::OpenWindow => {
                let Some(last_window) = app.windows.keys().last() else {
                    return Task::none();
                };

                window::position(*last_window)
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
                    .map(|id| Message::Window(WindowMessage::WindowOpened(id)))
            }
            Self::Page(id, x) => x.handle(app, id),
            Self::WindowClosed(id) => {
                app.windows.remove(&id);

                if app.windows.is_empty() {
                    iced::exit()
                } else {
                    Task::none()
                }
            }
            Self::WindowOpened(id) => {
                let window = Window::new(app.view_data.page_stack.pop().unwrap_or_default());
                let focus_input = operation::focus(format!("input-{id}"));

                app.windows.insert(id, window);

                focus_input
            }
        }
    }
}
