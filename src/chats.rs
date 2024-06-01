use color_art::Color;
use iced::widget::{column, mouse_area, text, container, scrollable};
use iced::{ Border, Element, Length, Theme};
use iced::Background; 
use crate::save::Chats;
use crate::utils::{darken_colour, lighten_colour};
use crate::{chats, Message};
use crate::THEME;

pub struct SideChats{
    chats : Vec<SideChat>,
}

impl SideChats{
    pub fn new(titles : Vec<String>) -> Self{
        let mut chats = Vec::new();

        titles.iter().enumerate().for_each(|(i, x)| chats.push(SideChat::new(x.clone(), i)));
        return Self{chats};
    }

    pub fn view(&self, chosen : Option<usize>) -> Element<Message>{
        let chats : Vec<Element<Message>> = self.chats.iter().enumerate().map(|(i, x)| if i == chosen.unwrap_or(usize::MAX) {x.view(true)} else {x.view(false)}).clone().collect();
        return scrollable(column(chats).spacing(2)).into();
    }
}

#[derive(Clone)]
struct SideChat{
    title : String,
    id : usize,
}

impl SideChat{
    pub fn new(title : String, id : usize) -> Self{
        return Self{
            title,
            id
        };
    }
    pub fn view(&self, chosen : bool) -> Element<Message>{
        let title = container(text(&self.title).size(16)).padding(5).width(Length::Fill);
        let mousea = mouse_area(title).on_press(Message::ChangeChat(self.id)).on_right_press(Message::RemoveChat(self.id));
        let bg = THEME.palette().background;
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
