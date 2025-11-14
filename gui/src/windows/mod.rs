// pub mod panes;
pub mod message;

use crate::{Application, Message, pages::Pages};
use iced::{Element, window};

#[derive(Debug, Clone)]
pub struct Window {
    pub page: Pages,
}

impl Window {
    pub fn new(page: Pages) -> Self {
        Self { page }
    }

    pub fn view<'a>(&'a self, app: &'a Application, id: window::Id) -> Element<'a, Message> {
        self.page.view(app, id)
    }
}
