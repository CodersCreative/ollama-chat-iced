use super::{message::OptionMessage, values::OptionKey};
use crate::{common::Id, style, utils::get_path_assets, ChatApp, Message};
use iced::{
    widget::{button, column, combo_box, container, row, svg, text},
    Element, Padding,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Options(pub String, pub Option<OptionKey>);

impl Options {
    pub fn new(model: String) -> Self {
        Self(model, None)
    }

    pub fn model(&self) -> &str {
        &self.0
    }

    pub fn set_model(&mut self, model: String) {
        self.0 = model;
    }

    pub fn key(&self) -> &Option<OptionKey> {
        &self.1
    }

    pub fn set_key(&mut self, key: Option<OptionKey>) {
        self.1 = key;
    }

    pub fn view<'a>(&'a self, key: Id, app: &'a ChatApp) -> Element<'a, Message> {
        let index = match app
            .options
            .get_model_options_index(self.model().to_string())
        {
            Some(x) => x,
            None => return text("Failed").into(),
        };
        self.view_with_index(app, index, key.clone(), &self.model())
    }

    pub fn view_with_index<'a>(
        &'a self,
        app: &'a ChatApp,
        index: usize,
        key: Id,
        model: &'a str,
    ) -> Element<'a, Message> {
        container(column![
            container(row![
                combo_box(&app.logic.combo_models, model, None, move |x| {
                    Message::Option(OptionMessage::ChangeModel(x), key)
                }),
                button(
                    svg(svg::Handle::from_path(get_path_assets(
                        "delete.svg".to_string()
                    )))
                    .style(style::svg::white)
                    .width(24.0)
                    .height(24.0),
                )
                .style(style::button::transparent_text)
                .on_press(Message::Option(OptionMessage::DeleteModel, key))
            ])
            .padding(10),
            container(app.options.0[index].view(app, key.clone()))
                .padding(Padding::default().left(20).right(20).top(5).bottom(5))
        ])
        .into()
    }
}
