use iced::{Element, window};

use crate::{
    Application, Message,
    pages::setup::{SetupMessage, SetupPage},
};

pub mod setup;

#[derive(Debug, Clone)]
pub enum Pages {
    Setup(SetupPage),
}

#[derive(Debug, Clone)]
pub enum PageMessage {
    Setup(SetupMessage),
}

impl PageMessage {
    pub fn handle(self, app: &mut Application, id: window::Id) -> iced::Task<Message> {
        match self {
            Self::Setup(x) => x.handle(app, id),
        }
    }
}

impl Default for Pages {
    fn default() -> Self {
        Self::Setup(SetupPage::default())
    }
}

impl Pages {
    pub fn view<'a>(&'a self, app: &'a Application, id: window::Id) -> Element<'a, Message> {
        match self {
            Pages::Setup(x) => x.view(app, id),
        }
    }
}
