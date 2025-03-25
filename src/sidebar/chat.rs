use std::time::SystemTime;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, text};
use iced::{ Element, Length, Padding};
use crate::{style, Message};

#[derive(Clone)]
pub struct Chat{
    pub title : String,
    pub time : SystemTime, 
    pub id : usize,

}

impl Chat{
    pub fn new(title : String, time: SystemTime, id : usize) -> Self{
        return Self{
            title,
            time,
            id,
        };
    }
    pub fn view(&self, chosen : bool) -> Element<Message>{
        let style = match chosen{
            true => style::button::chosen_chat,
            false => style::button::not_chosen_chat,
        };
        
        button(
            text(&self.title).align_x(Horizontal::Center).align_y(Vertical::Center).width(Length::Fill).size(20)
        )
        .style(style)
        .on_press(Message::ChangeChat(self.id))
        .width(Length::Fill).padding(Padding::from(10))
        .into()
    }
}
