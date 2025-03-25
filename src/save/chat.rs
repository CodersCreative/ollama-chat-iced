use iced::{alignment::{Horizontal, Vertical},widget::{button, column, container, markdown, row, text}, Theme};
use iced::{Element, Length};
use serde::{Deserialize, Serialize};
use crate::{style::{self}, Message};

#[derive(Serialize, Deserialize, Debug,Clone, PartialEq, Default)]
pub struct Chat{
    pub name: String,
    pub message: String,
}


impl Chat{
    pub fn new(name : &str, messasge : &str) -> Self{
        return Self{
            name: name.to_string(),
            message: messasge.to_string(),
        }
    }

    pub fn generate_mk(text : &str) -> Vec<markdown::Item>{
        markdown::parse(text).collect::<Vec<markdown::Item>>()
    }

    pub fn view_mk<'a>(&'a self, markdown : &'a Vec<markdown::Item>, theme : &Theme) -> Element<'a, Message>{
        markdown::view(markdown, markdown::Settings::default(), markdown::Style::from_palette(theme.palette()))
            .map(Message::URLClicked)
            .into()
    }

    pub fn view<'a>(&'a self, markdown : &'a Vec<markdown::Item>, theme: &Theme) -> Element<'a, Message> {
        let is_ai = self.name != "User";
        
        let style = match is_ai{
            true => style::container::chat_ai,
            false => style::container::chat,
        };

        let copy = button(text("Copy").size(16).align_x(Horizontal::Right).align_y(Vertical::Center)).width(Length::Fill).style(style::button::transparent_text).on_press(Message::SaveToClipboard(self.message.clone()));

        let name = container(
            row![
                text(&self.name).size(16).align_x(Horizontal::Left).align_y(Vertical::Center),
                copy,
            ]
        ).style(style)
        .width(Length::Fill).padding(3);
        
        let mark = container(self.view_mk(markdown, theme)).padding(20);

        let style = match is_ai{
            true => style::container::chat_back_ai,
            false => style::container::chat_back,
        };
        container(column![
            name,
            mark,
        ].width(Length::Fill)).style(style).width(Length::FillPortion(5)).into()
    }
}
