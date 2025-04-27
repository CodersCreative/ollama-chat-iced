use super::chats::ChatsMessage;
use crate::{
    chats::Chats,
    common::Id,
    style::{self},
    utils::{convert_image, get_path_assets},
    Message,
};
use derive_builder::Builder;
use getset::{Getters, Setters};
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container, image, markdown, row,
        scrollable::{self, Direction, Scrollbar},
        svg, text, text_editor, Button,
    },
    Padding, Theme,
};
use iced::{Element, Length};
use ollama_rs::generation::{chat::ChatMessage, tools::ToolCall};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::SystemTime};

#[derive(Builder, Serialize, Deserialize, Debug, Clone, Getters, Setters)]
pub struct Chat {
    #[getset(get = "pub", set = "pub")]
    #[builder(default = "Role::User")]
    role: Role,

    #[getset(get = "pub", set = "pub")]
    content: String,

    #[getset(get = "pub", set = "pub")]
    #[builder(default = "Vec::new()")]
    images: Vec<PathBuf>,

    #[getset(get = "pub", set = "pub")]
    #[builder(default = "Vec::new()")]
    tools: Vec<ToolCall>,

    #[getset(get = "pub", set = "pub")]
    #[builder(default = "SystemTime::now()")]
    timestamp: SystemTime,
}

impl Chat {
    pub fn update_content(&mut self, f: fn(&mut String)) {
        f(&mut self.content);
    }

    pub fn add_to_content(&mut self, text: &str) {
        self.content.push_str(text);
    }
}

impl PartialEq for Chat {
    fn eq(&self, other: &Self) -> bool {
        self.role == other.role && self.content == other.content
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum Role {
    #[default]
    User,
    AI,
}

impl From<usize> for Role {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::User,
            _ => Self::AI,
        }
    }
}

impl Into<usize> for Role {
    fn into(self) -> usize {
        match self {
            Self::User => 0,
            _ => 1,
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Self::AI => "Ai",
            Self::User => "User",
        };
        write!(f, "{}", output)
    }
}
impl Into<ChatMessage> for &Chat {
    fn into(self) -> ChatMessage {
        let mut message = match self.role {
            Role::User => ChatMessage::user(self.content().to_string()),
            Role::AI => ChatMessage::assistant(self.content().to_string()),
        };

        message.images = match self.images.len() > 0 {
            true => Some(
                self.images
                    .iter()
                    .map(|x| convert_image(x).unwrap())
                    .collect(),
            ),
            false => None,
        };

        message.tool_calls = self.tools.clone();

        message
    }
}

impl Chat {
    pub fn new(role: &Role, message: &str, images: Vec<PathBuf>, tools: Vec<ToolCall>) -> Self {
        return Self {
            role: role.clone(),
            content: message.to_string(),
            images,
            tools,
            timestamp: SystemTime::now(),
        };
    }

    pub fn generate_mk(text: &str) -> Vec<markdown::Item> {
        markdown::parse(text).collect::<Vec<markdown::Item>>()
    }

    pub fn view_mk<'a>(
        &'a self,
        markdown: &'a Vec<markdown::Item>,
        theme: &Theme,
    ) -> Element<'a, Message> {
        markdown::view(
            markdown,
            markdown::Settings::default(),
            markdown::Style::from_palette(theme.palette()),
        )
        .map(Message::URLClicked)
        .into()
        //markdown::view_with(markdown, theme, &style::markdown::CustomViewer).into()
    }

    pub fn view_editing<'a>(
        &'a self,
        id: Id,
        content: &'a text_editor::Content,
    ) -> Element<'a, Message> {
        let is_ai = self.role == Role::AI;

        let style = match is_ai {
            true => style::container::chat_ai,
            false => style::container::chat,
        };

        let btn = |img: &str| -> Button<Message> {
            button(
                svg(svg::Handle::from_path(get_path_assets(img.to_string())))
                    .style(style::svg::white)
                    .width(16.0)
                    .height(16.0),
            )
            .style(style::button::transparent_text)
        };

        let name = container(
            row![
                text(match self.role {
                    Role::User => "User",
                    Role::AI => "Assistant",
                })
                .size(16)
                .align_x(Horizontal::Left)
                .align_y(Vertical::Center)
                .width(Length::Fill),
                btn("save.svg").on_press(Message::Chats(ChatsMessage::SaveEdit, id.clone())),
                btn("close.svg").on_press(Message::Chats(ChatsMessage::CancelEdit, id.clone())),
            ]
            .spacing(10),
        )
        .style(style)
        .width(Length::Fill)
        .padding(3);

        let images = container(
            scrollable::Scrollable::new(
                row(self.images.iter().map(|x| {
                    button(image(image::Handle::from_path(x)).height(Length::Fixed(200.0)))
                        .style(style::button::transparent_text)
                        .into()
                }))
                .align_y(Vertical::Center)
                .spacing(10),
            )
            .direction(Direction::Horizontal(Scrollbar::new())),
        )
        .padding(Padding::from([0, 20]))
        .style(style::container::bottom_input_back);

        let editor = container(
            text_editor(content)
                .placeholder("Type your message here...")
                .on_action(move |action| Message::Chats(ChatsMessage::EditAction(action), id))
                .padding(Padding::from(20))
                .size(20)
                .style(style::text_editor::input),
        )
        .padding(20);

        let style = match is_ai {
            true => style::container::chat_back_ai,
            false => style::container::chat_back,
        };
        container(column![name, images, editor,].width(Length::Fill))
            .style(style)
            .width(Length::FillPortion(5))
            .into()
    }

    pub fn view<'a>(
        &'a self,
        id: &Id,
        markdown: &'a Vec<markdown::Item>,
        theme: &Theme,
    ) -> Element<'a, Message> {
        let is_ai = self.role == Role::AI;

        let style = match is_ai {
            true => style::container::chat_ai,
            false => style::container::chat,
        };

        let btn = |img: &str| -> Button<Message> {
            button(
                svg(svg::Handle::from_path(get_path_assets(img.to_string())))
                    .style(style::svg::white)
                    .width(16.0)
                    .height(16.0),
            )
            .style(style::button::transparent_text)
        };

        let name = container(
            row![
                text(match self.role {
                    Role::User => "User",
                    Role::AI => "Assistant",
                })
                .size(16)
                .align_x(Horizontal::Left)
                .align_y(Vertical::Center)
                .width(Length::Fill),
                btn("edit.svg").on_press(Message::Chats(
                    ChatsMessage::Edit(self.content().clone()),
                    id.clone()
                )),
                btn("restart.svg").on_press(Message::Chats(ChatsMessage::Regenerate, id.clone())),
                btn("copy.svg").on_press(Message::SaveToClipboard(self.content().to_string())),
            ]
            .spacing(10),
        )
        .style(style)
        .width(Length::Fill)
        .padding(3);

        let images = container(
            scrollable::Scrollable::new(
                row(self.images.iter().map(|x| {
                    button(image(image::Handle::from_path(x)).height(Length::Fixed(200.0)))
                        .style(style::button::transparent_text)
                        .into()
                }))
                .align_y(Vertical::Center)
                .spacing(10),
            )
            .direction(Direction::Horizontal(Scrollbar::new())),
        )
        .padding(Padding::from([0, 20]))
        .style(style::container::bottom_input_back);
        let mark = container(self.view_mk(markdown, theme)).padding(20);

        let style = match is_ai {
            true => style::container::chat_back_ai,
            false => style::container::chat_back,
        };
        container(column![name, images, mark,].width(Length::Fill))
            .style(style)
            .width(Length::FillPortion(5))
            .into()
    }
}
