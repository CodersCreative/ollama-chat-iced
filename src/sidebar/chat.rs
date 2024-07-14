use iced::widget::{mouse_area, text, container};
use iced::{ Element, Length};
use crate::utils::darken_colour;
use crate::Message;

#[derive(Clone)]
pub struct Chat{
    title : String,
    id : usize,
}

impl Chat{
    pub fn new(title : String, id : usize) -> Self{
        return Self{
            title,
            id
        };
    }
    pub fn view(&self, chosen : bool, bg : iced::Color) -> Element<Message>{
        let title = container(text(&self.title).size(16)).padding(5).width(Length::Fill);
        let mousea = mouse_area(title).on_press(Message::ChangeChat(self.id)).on_right_press(Message::RemoveChat(self.id));
        let bg = bg.clone();

        let bg = match chosen{
            true => bg,
            false => darken_colour(bg, 0.015),
        };

        let style = container(mousea).width(Length::Fill).style(container::Appearance{
            background : Some(iced::Background::Color(bg)),
            ..Default::default()
        });

        style.into()
    }
}
