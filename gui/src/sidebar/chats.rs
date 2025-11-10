use std::time::SystemTime;

use super::chat::SideChat;
use crate::common::Id;
use crate::{ChatApp, Message};
use iced::widget::{column, scrollable};
use iced::Element;

#[derive(Clone)]
pub struct SideChats {
    pub chats: Vec<SideChat>,
}

impl SideChats {
    pub fn new(titles: Vec<(Id, String, SystemTime)>) -> Self {
        return Self {
            chats: titles
                .into_iter()
                .map(|(id, title, time)| SideChat::new(id, title, time))
                .collect(),
        };
    }

    pub fn new_with_chats(chats: Vec<SideChat>) -> Self {
        Self { chats }
    }

    pub fn view<'a>(&'a self, app: &ChatApp) -> Element<'a, Message> {
        let chats: Vec<Element<Message>> = self.chats.iter().map(|x| x.view(app)).clone().collect();
        return scrollable(column(chats).spacing(2)).into();
    }
}
