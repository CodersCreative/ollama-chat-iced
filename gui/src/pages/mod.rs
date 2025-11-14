use iced::{Element, window};

use crate::{Application, Message, pages::setup::SetupPage};

pub mod setup;

#[derive(Debug, Clone)]
pub enum Pages {
    Setup(SetupPage),
}

impl Default for Pages {
    fn default() -> Self {
        Self::Setup(SetupPage::default())
    }
}

impl Pages {
    pub fn view<'a>(&'a self, app: &'a Application, id: &window::Id) -> Element<'a, Message> {
        match self {
            Pages::Setup(x) => x.view(app),
        }
    }
}
