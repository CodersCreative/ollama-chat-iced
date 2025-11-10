use iced::{widget::text, Element};

use crate::{common::Id, ChatApp, Message};

#[derive(Debug, Clone, Default)]
pub struct Settings {}

impl Settings {
    pub fn new(app: &ChatApp) -> Self {
        Self {}
    }

    pub fn view<'a>(&'a self, key: Id, app: &'a ChatApp) -> Element<'a, Message> {
        text("Hello, World!").into()
    }
}
