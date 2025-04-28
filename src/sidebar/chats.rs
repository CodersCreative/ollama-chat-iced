use super::chat::SideChat;
use crate::common::Id;
use crate::{ChatApp, Message};
use iced::widget::{column, scrollable};
use iced::Element;
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Clone)]
pub struct SideChats {
    pub chats: HashMap<Id, SideChat>,
}

impl SideChats {
    pub fn new(titles: Vec<(Id, String, SystemTime)>) -> Self {
        let mut chats = HashMap::new();

        titles.iter().for_each(|(id, title, time)| {
            chats.insert(id.clone(), SideChat::new(title.clone(), time.clone()));
        });
        return Self { chats };
    }

    pub fn new_with_chats(chats: HashMap<Id, SideChat>) -> Self {
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
