use iced::{Size, Task, Vector, widget::operation, window};

use crate::{
    Application, Message,
    pages::{
        PageMessage, Pages,
        home::{HomePage, panes::PaneMessage},
        setup::SetupPage,
    },
    windows::Window,
};

#[derive(Debug, Clone)]
pub enum WindowMessage {
    OpenWindow,
    CloseWindow(window::Id),
    Page(window::Id, PageMessage),
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    Resize(window::Id, Size<f32>),
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
            Self::CloseWindow(id) => window::close(id),
            Self::Page(id, x) => x.handle(app, id),
            Self::Resize(id, size) => {
                let win = app.windows.get_mut(&id).unwrap();
                win.size = size;
                Task::none()
            }
            Self::WindowClosed(id) => {
                app.windows.remove(&id);

                if app.windows.is_empty() {
                    iced::exit()
                } else {
                    Task::none()
                }
            }
            Self::WindowOpened(id) => {
                let mut is_chat = false;
                let window = Window::new(if let Some(x) = app.view_data.page_stack.pop() {
                    x
                } else if app.cache.client_settings.default_provider.is_none() {
                    let mut setup = SetupPage::default();
                    setup.instance_url = app.cache.client_settings.instance_url.clone();
                    Pages::Setup(setup)
                } else {
                    is_chat = true;
                    Pages::Home(HomePage::new())
                });
                let focus_input = operation::focus(format!("input-{id}"));

                app.windows.insert(id, window);

                Task::batch([
                    window::size(id).map(move |x| Message::Window(WindowMessage::Resize(id, x))),
                    focus_input,
                    if is_chat {
                        let Pages::Home(pane) = &app.windows.get(&id).unwrap().page else {
                            panic!()
                        };
                        let pane = pane.panes.panes.panes.first_key_value().unwrap().0.clone();
                        PaneMessage::handle_new_chat(app, id, pane)
                    } else {
                        Task::none()
                    },
                ])
            }
        }
    }
}
