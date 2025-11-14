use iced::window;

use crate::pages::Pages;

#[derive(Debug, Clone)]
pub struct Window {
    page: Pages,
}

#[derive(Debug, Clone)]
pub enum WindowMessage {
    OpenWindow,
    WindowOpened(window::Id, Pages),
    WindowClosed(window::Id),
}

impl Window {
    pub fn new(page: Pages) -> Self {
        Self { page }
    }
}
