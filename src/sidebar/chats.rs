use super::chat::Chat;
use crate::common::Id;
use crate::{ChatApp, Message};
use iced::widget::{column, scrollable};
use iced::Element;
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Clone)]
pub struct Chats {
    pub chats: HashMap<Id, Chat>,
}

impl Chats {
    pub fn new(titles: Vec<(Id, String, SystemTime)>) -> Self {
        let mut chats = HashMap::new();

        titles.iter().for_each(|(id, title, time)| {
            chats.insert(id.clone(), Chat::new(title.clone(), time.clone()));
        });
        return Self { chats };
    }

    pub fn new_with_chats(chats: HashMap<Id, Chat>) -> Self {
        Self { chats }
    }

    pub fn view(&self, app: &ChatApp) -> Element<Message> {
        let chats: Vec<Element<Message>> = self
            .chats
            .iter()
            .map(|(i, x)| x.view(app, i))
            .clone()
            .collect();
        return scrollable(column(chats).spacing(2)).into();
    }
}
