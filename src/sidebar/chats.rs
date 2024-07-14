use iced::widget::{column, scrollable};
use iced::Element;
use crate::Message;
use super::chat::Chat;

pub struct Chats{
    chats : Vec<Chat>,
}

impl Chats{
    pub fn new(titles : Vec<String>) -> Self{
        let mut chats = Vec::new();

        titles.iter().enumerate().for_each(|(i, x)| chats.push(Chat::new(x.clone(), i)));
        return Self{chats};
    }

    pub fn view(&self, chosen : Option<usize>, chat_bg : iced::Color) -> Element<Message>{
        let chats : Vec<Element<Message>> = self.chats.iter().enumerate().map(|(i, x)| x.view(i == chosen.unwrap_or(usize::MAX), chat_bg)).clone().collect();
        return scrollable(column(chats).spacing(2)).into();
    }
}
