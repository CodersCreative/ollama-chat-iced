use std::str::FromStr;

use iced::{alignment::{Horizontal, Vertical}, widget::{button, column, container, keyed_column, row, scrollable, svg, text, text_editor, text_input, vertical_space, Container, Space}, Element, Length, Renderer, Theme};
use ollama_rs::IntoUrlSealed;
use url::Url;

use crate::{common::Id, save::chats::ChatsMessage, style, utils::get_path_assets, ChatApp, Message};

use super::{message::PromptsMessage, Prompt, SavedPrompts};

impl SavedPrompts{
    pub fn input_view(&self, input : &str, id: &Id) -> Element<Message>{
        if let Some(input) = get_command_input(input){
            return keyed_column(self.search(input).unwrap().iter().enumerate().map(|(_, prompt)| {
                (
                    0,
                    button(text(prompt.command.clone()))
                    .on_press(Message::Chats(ChatsMessage::PickedPrompt(prompt.command.clone()), id.clone()))
                    .style(style::button::transparent_text).into(),
                )
            }))
            .spacing(10)
            .into();
        }
        Space::with_height(0).into()
    }
}

fn get_command_input(input : &str) -> Option<&str>{
    if let Some(split) = input.split_whitespace().last(){
        if split.contains("/"){
            return Some(split.trim_start_matches("/"));
        }
    }

    None
}

#[derive(Default)]
pub struct Prompts{pub expand : Option<String>, pub input : String, pub prompts : Vec<Prompt>, pub edit : Edit}

#[derive(Default)]
pub struct Edit{
    pub content : text_editor::Content,
    pub title : String,
    pub command : String,
    pub og_command : String
}

impl From<Prompt> for Edit{
    fn from(value: Prompt) -> Self {
        Self{
            content: text_editor::Content::with_text(&value.content),
            title : value.title.clone(),
            command : value.command.clone(),
            og_command : value.command.clone(),
        }
    }
}

// impl Into<Edit> for Prompt{
//     fn into(self) -> Edit {
//         Edit{
//             content: text_editor::Content::with_text(&self.content),
//             title : self.title.clone(),
//             command : self.command.clone(),
//         }
//     }
// }
// struct Prompts{
//     input : text_editor::Content,
// }

impl Prompts{
    pub fn new(app: &ChatApp) -> Self {
        Self{
            expand: None,
            input : String::new(),
            prompts : app.prompts.prompts.iter().map(|x| x.1.clone()).collect(),
            edit: Edit::default(),
        }
    }
    pub fn view_prompts<'a>(&'a self, app: &'a ChatApp, id: Id) -> Element<'a, Message> {
        keyed_column(self.prompts.iter().enumerate().map(|(_i, prompt)| {
            let mut expand = false;

            if let Some(x) = &self.expand {
                expand = x == &prompt.command;
            }

            (0, prompt.view(app, id.clone(), expand, &self.edit))
        }))
        .spacing(10)
        .into()
    }

    pub fn view<'a>(&'a self, key: Id, app: &'a ChatApp) -> Element<'a, Message> {
        let input = text_input::<Message, Theme, Renderer>("Enter your message", &self.input)
            .on_input(move |x| Message::Prompts(PromptsMessage::Input(x), key.clone()))
            .on_submit(Message::Prompts(PromptsMessage::Search, key))
            .size(16)
            .style(style::text_input::input)
            .width(Length::Fill);

        let btn = |file: &str| -> button::Button<'a, Message, Theme, Renderer> {
            button(
                svg(svg::Handle::from_path(get_path_assets(file.to_string())))
                    .style(style::svg::primary)
                    .width(Length::Fixed(32.0)),
            )
            .style(style::button::chosen_chat)
            .width(Length::Fixed(48.0))
        };

        let input = row![
            input,
            btn("upload.svg").on_press(Message::Prompts(PromptsMessage::Upload, key.clone())),
            btn("add.svg").on_press(Message::Prompts(PromptsMessage::Add, key.clone()))
        ];

        let help_text = button(text("Format your variables using brackets like this: {{variable}}. Make sure to enclose them with {{ and }}.
Utilize {{CLIPBOARD}} variable to have them replaced with clipboard content.\nPrompts can also be gotten from open-webui.")
            .color(app.theme().palette().text)
            .size(10)
            .width(Length::Fill)
            .align_y(Vertical::Center)
            .align_x(Horizontal::Left)).style(style::button::not_chosen_chat).padding(6).on_press(Message::URLClicked(Url::from_str("https://openwebui.com/prompts").unwrap()));
        
        container(column![
            input,
            scrollable::Scrollable::new(self.view_prompts(app, key.clone())).width(Length::Fill),
            vertical_space(),
            help_text,
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
    }
}
