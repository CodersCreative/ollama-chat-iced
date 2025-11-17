use iced::{Element, Task, window};

use crate::{
    Application, Message,
    pages::{
        home::{HomePage, message::HomeMessage},
        setup::{SetupMessage, SetupPage},
    },
};

pub mod home;
pub mod setup;

#[derive(Debug, Clone)]
pub enum Pages {
    Setup(SetupPage),
    Home(HomePage),
}

#[derive(Debug, Clone)]
pub enum PageMessage {
    Setup(SetupMessage),
    Home(HomeMessage),
}

impl PageMessage {
    pub fn handle(self, app: &mut Application, id: window::Id) -> Task<Message> {
        match self {
            Self::Setup(x) => x.handle(app, id),
            Self::Home(x) => x.handle(app, id),
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
            Pages::Home(x) => x.view(app, id),
        }
    }
}
