use crate::{utils::convert_image, Message};
use derive_builder::Builder;
use getset::{Getters, Setters};
use iced::Element;
use iced::{widget::markdown, Theme};
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Role {
    #[default]
    User,
    AI,
    System,
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
            Self::System => "System",
        };
        write!(f, "{}", output)
    }
}
impl Into<ChatMessage> for &Chat {
    fn into(self) -> ChatMessage {
        let mut message = match self.role {
            Role::User => ChatMessage::user(self.content().to_string()),
            Role::AI => ChatMessage::assistant(self.content().to_string()),
            Role::System => ChatMessage::system(self.content().to_string()),
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
}
