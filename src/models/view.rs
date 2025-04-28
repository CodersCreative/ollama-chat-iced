use iced::{
    widget::{column, container, keyed_column, scrollable, text_input},
    Element, Length, Renderer, Theme,
};

use crate::{common::Id, style, ChatApp, Message};

use super::{message::ModelsMessage, model::ModelInfo};

#[derive(Debug, Clone)]
pub struct Models(pub Option<String>, pub String, pub Vec<ModelInfo>);

impl Models {
    pub fn new(app: &ChatApp) -> Self {
        Self(None, String::new(), app.model_info.models.clone())
    }

    pub fn view_models<'a>(&'a self, app: &'a ChatApp, id: Id) -> Element<'a, Message> {
        keyed_column(self.2.iter().enumerate().map(|(_i, model)| {
            let mut expand = false;

            if let Some(x) = &self.0 {
                expand = x == &model.name;
            }
            (0, model.view(app, id.clone(), expand))
        }))
        .spacing(10)
        .into()
    }

    pub fn view<'a>(&'a self, key: Id, app: &'a ChatApp) -> Element<'a, Message> {
        let input = text_input::<Message, Theme, Renderer>("Enter your message", &self.1)
            .on_input(move |x| Message::Models(ModelsMessage::Input(x), key.clone()))
            .on_submit(Message::Models(ModelsMessage::Search, key))
            .size(16)
            .style(style::text_input::input)
            .width(Length::Fill);

        container(column![
            input,
            scrollable::Scrollable::new(self.view_models(app, key.clone())).width(Length::Fill)
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
    }
}
