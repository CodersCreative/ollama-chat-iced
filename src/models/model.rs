use super::message::ModelsMessage;
use crate::{common::Id, style, ChatApp, Message};
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{button, column, container, markdown, row, text},
    Element, Length,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TempInfo {
    url: String,
    tags: Vec<Vec<String>>,
    author: String,
    categories: Vec<String>,
    languages: Vec<String>,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ModelInfo {
    pub name: String,
    pub url: String,
    pub tags: Vec<Vec<String>>,
    pub author: String,
    pub categories: Vec<String>,
    pub languages: Vec<String>,
}

impl ModelInfo {
    pub fn view<'a>(&'a self, app: &'a ChatApp, id: Id, expand: bool) -> Element<'a, Message> {
        let mut widgets: Vec<Element<Message>> = Vec::new();

        widgets.push(
            button(
                text(self.name.clone())
                    .color(app.theme().palette().primary)
                    .size(24)
                    .width(Length::Fill)
                    .align_y(Vertical::Center)
                    .align_x(Horizontal::Left),
            )
            .padding(0)
            .style(style::button::transparent_back)
            .on_press(Message::Models(
                ModelsMessage::Expand(self.name.clone()),
                id,
            ))
            .into(),
        );

        widgets.push(
            text(&self.author)
                .color(app.theme().palette().danger)
                .size(20)
                .width(Length::Fill)
                .align_y(Vertical::Center)
                .align_x(Horizontal::Left)
                .into(),
        );

        if let Some(x) = app.model_info.descriptions.get(&self.name) {
            widgets.push(
                text(x)
                    .color(app.theme().palette().text)
                    .size(16)
                    .width(Length::Fill)
                    .align_y(Vertical::Center)
                    .align_x(Horizontal::Left)
                    .into(),
            );
        }

        if expand {
            widgets.push(
                button(text(&self.url).size(16))
                    .style(style::button::chosen_chat)
                    .on_press(Message::URLClicked(
                        markdown::Url::from_str(&self.url).unwrap(),
                    ))
                    .into(),
            );
            for tag in &self.tags {
                widgets.push(
                    button(row![
                        text(tag[0].clone())
                            .align_x(Horizontal::Center)
                            .align_y(Vertical::Center)
                            .width(Length::Fill)
                            .size(16),
                        text(tag[1].clone())
                            .align_x(Horizontal::Center)
                            .align_y(Vertical::Center)
                            .width(Length::Fill)
                            .size(16)
                    ])
                    .style(style::button::not_chosen_chat)
                    .on_press(Message::Pull(format!("{}:{}", self.name, tag[0])))
                    .width(Length::Fill)
                    .padding(10)
                    .into(),
                );
            }
        }

        container(column(widgets).padding(10))
            .padding(5)
            .style(style::container::side_bar)
            .into()
    }
}

impl Into<ModelInfo> for TempInfo {
    fn into(self) -> ModelInfo {
        ModelInfo {
            name: String::new(),
            url: self.url,
            tags: self.tags,
            author: self.author,
            categories: self.categories,
            languages: self.languages,
        }
    }
}
