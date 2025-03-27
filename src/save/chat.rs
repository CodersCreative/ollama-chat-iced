use std::path::PathBuf;

use iced::{alignment::{Horizontal, Vertical}, widget::{button, column, container, horizontal_space, image, markdown, row, scrollable::{self, Direction, Scrollbar}, text}, Padding, Theme};
use iced::{Element, Length};
use ollama_rs::generation::chat::ChatMessage;
use serde::{Deserialize, Serialize};
use crate::{style::{self}, utils::convert_image, Message};

#[derive(Serialize, Deserialize, Debug,Clone, PartialEq, Default)]
pub struct Chat{
    pub role: Role,
    pub message: String,
    pub images: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug,Clone, PartialEq, Default)]
pub enum Role {
    #[default]
    User,
    AI,
}

impl Into<ChatMessage> for &Chat{
    fn into(self) -> ChatMessage {
        let mut message = match self.role{
            Role::User => ChatMessage::user(self.message.clone()),
            Role::AI => ChatMessage::assistant(self.message.clone()),
        };

        message.images = match self.images.len() > 0{
            true => Some(self.images.iter().map(|x| {
                convert_image(x).unwrap()
            }).collect()),
            false => None
        };

        message
    }
}

impl Chat{
    pub fn new(role : &Role, messasge : &str, images : Vec<PathBuf>) -> Self{
        return Self{
            role: role.clone(),
            message: messasge.to_string(),
            images,
        }
    }

    pub fn generate_mk(text : &str) -> Vec<markdown::Item>{
        markdown::parse(text).collect::<Vec<markdown::Item>>()
    }

    pub fn view_mk<'a>(&'a self, markdown : &'a Vec<markdown::Item>, theme : &Theme) -> Element<'a, Message>{
        //markdown::view(markdown, markdown::Settings::default(), markdown::Style::from_palette(theme.palette()))
        //    .map(Message::URLClicked)
        //    .into()

        markdown::view_with(markdown, theme, &style::markdown::CustomViewer).into()
    }

    pub fn view<'a>(&'a self, markdown : &'a Vec<markdown::Item>, theme: &Theme) -> Element<'a, Message> {
        let is_ai = self.role == Role::AI;
        
        let style = match is_ai{
            true => style::container::chat_ai,
            false => style::container::chat,
        };

        let copy = button(text("Copy").size(16).align_x(Horizontal::Right).align_y(Vertical::Center)).style(style::button::transparent_text).on_press(Message::SaveToClipboard(self.message.clone()));

        let regenerate = button(text("Regen").size(16).align_x(Horizontal::Right).align_y(Vertical::Center)).style(style::button::transparent_text).on_press(Message::Regenerate);
        
        let name = container(
            row![
                text(match self.role{
                    Role::User => "User",
                    Role::AI => "Assistant"
                }).size(16).align_x(Horizontal::Left).align_y(Vertical::Center).width(Length::Fill),
                regenerate,
                copy,
            ].spacing(10)
        ).style(style)
        .width(Length::Fill).padding(3);
        
        let images = container(
            scrollable::Scrollable::new(row(self.images.iter().map(|x| {
               button(image(image::Handle::from_path(x)).height(Length::Fixed(100.0))).style(style::button::transparent_text).on_press(Message::RemoveImage(x.clone())).into() 
            })).align_y(Vertical::Center).spacing(10)).direction(Direction::Horizontal(Scrollbar::new()))
        ).padding(Padding::from([0, 20])).style(style::container::bottom_input_back);
        let mark = container(self.view_mk(markdown, theme)).padding(20);

        let style = match is_ai{
            true => style::container::chat_back_ai,
            false => style::container::chat_back,
        };
        container(column![
            name,
            images,
            mark,
        ].width(Length::Fill)).style(style).width(Length::FillPortion(5)).into()
    }
}
