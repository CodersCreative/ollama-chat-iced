use std::time::SystemTime;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, row, text};
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
        
        let title = button(
            text(&self.title).align_x(Horizontal::Center).align_y(Vertical::Center).width(Length::Fill).size(20)
        )
        .style(style)
        .on_press(Message::ChangeChat(self.id))
        .width(Length::FillPortion(7)).padding(Padding::from(10));

        let remove = button(
            text("x").align_x(Horizontal::Center).align_y(Vertical::Center).width(Length::Fill).size(20)
        )
        .style(style)
        .on_press(Message::RemoveChat(self.id))
        .width(Length::FillPortion(1)).padding(Padding::from(10));
        row![
            title,
            remove,
        ].padding(5).into()
    }
}
