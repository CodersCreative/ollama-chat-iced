use super::chat::Chat;
use crate::{ChatApp, Message};
use iced::widget::{column, scrollable};
use iced::Element;
use std::time::SystemTime;

#[derive(Clone)]
pub struct Chats {
    pub chats: Vec<Chat>,
}

impl Chats {
    pub fn new(titles: Vec<(String, SystemTime)>) -> Self {
        let mut chats = Vec::new();

        titles
            .iter()
            .enumerate()
            .for_each(|(i, (x, y))| chats.push(Chat::new(x.clone(), y.clone(), i)));
        return Self { chats };
    }

    pub fn new_with_chats(chats: Vec<Chat>) -> Self {
        Self { chats }
    }

    pub fn view(&self, app: &ChatApp) -> Element<Message> {
        let chats: Vec<Element<Message>> = self.chats.iter().map(|x| x.view(app)).clone().collect();
        return scrollable(column(chats).spacing(2)).into();
    }
}
