pub mod view;

use crate::utils::convert_audio;
use crate::{utils::convert_image, Message};
use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestFunctionMessageArgs,
    ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartImageArgs,
    ChatCompletionRequestMessageContentPartTextArgs, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, ChatCompletionRequestUserMessageContentPart,
    ImageUrlArgs, InputAudio, InputAudioFormat,
};
use derive_builder::Builder;
use getset::{Getters, Setters};
use iced::Element;
use iced::{widget::markdown, Theme};
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
    images: Vec<FileType>,

    #[getset(get = "pub", set = "pub")]
    #[builder(default = "Vec::new()")]
    audio: Vec<AudioType>,

    #[getset(get = "pub", set = "pub")]
    #[builder(default = "SystemTime::now()")]
    timestamp: SystemTime,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FileType {
    Url(String),
    Base64(String),
    Path(PathBuf),
}

impl Into<PathBuf> for &FileType {
    fn into(self) -> PathBuf {
        match self {
            FileType::Path(x) => x.clone(),
            _ => panic!("Expected File::Path"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioType {
    file_type: FileType,
    audio_format: InputAudioFormat,
}

impl Into<ImageUrlArgs> for &FileType {
    fn into(self) -> ImageUrlArgs {
        match self {
            FileType::Url(x) => ImageUrlArgs::default().url(x).clone(),
            FileType::Base64(x) => ImageUrlArgs::default().url(x).clone(),
            FileType::Path(x) => ImageUrlArgs::default()
                .url(convert_image(x).unwrap())
                .clone(),
        }
    }
}

impl Into<InputAudio> for &AudioType {
    fn into(self) -> InputAudio {
        match &self.file_type {
            FileType::Base64(x) => InputAudio {
                data: x.clone(),
                format: self.audio_format.clone(),
            },
            FileType::Path(x) => InputAudio {
                data: convert_audio(x).unwrap(),
                format: self.audio_format.clone(),
            },
            _ => todo!(),
        }
    }
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
    Function,
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
            Self::AI => "AI",
            Self::User => "User",
            Self::System => "System",
            Self::Function => "Tool",
        };
        write!(f, "{}", output)
    }
}
impl Into<ChatCompletionRequestMessage> for &Chat {
    fn into(self) -> ChatCompletionRequestMessage {
        match self.role {
            Role::User => {
                let mut parts: Vec<ChatCompletionRequestUserMessageContentPart> =
                    vec![ChatCompletionRequestMessageContentPartTextArgs::default()
                        .text(self.content.to_string())
                        .build()
                        .unwrap()
                        .into()];

                for image in self.images.iter() {
                    parts.push(
                        ChatCompletionRequestMessageContentPartImageArgs::default()
                            .image_url(Into::<ImageUrlArgs>::into(image).build().unwrap())
                            .build()
                            .unwrap()
                            .into(),
                    );
                }

                ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(parts)
                        .build()
                        .unwrap(),
                )
            }
            Role::AI => ChatCompletionRequestMessage::Assistant(
                ChatCompletionRequestAssistantMessageArgs::default()
                    .content(self.content.to_string())
                    .build()
                    .unwrap(),
            ),
            Role::System => ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(self.content.to_string())
                    .build()
                    .unwrap(),
            ),
            Role::Function => ChatCompletionRequestMessage::Function(
                ChatCompletionRequestFunctionMessageArgs::default()
                    .content(self.content.to_string())
                    .build()
                    .unwrap(),
            ),
        }
    }
}

impl Chat {
    pub fn new(role: &Role, message: &str, images: Vec<FileType>, audio: Vec<AudioType>) -> Self {
        return Self {
            role: role.clone(),
            content: message.to_string(),
            images,
            audio,
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
