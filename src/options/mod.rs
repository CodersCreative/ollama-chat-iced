pub mod convert;
pub mod doc;
pub mod message;
pub mod values;
pub mod view;

use crate::{common::Id, style, utils::get_path_settings, ChatApp, Message};
use doc::DOCS;
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{button, column, container, row, scrollable, text, text_input, toggler},
    Element, Length,
};
use message::OptionMessage;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{fs::File, io::Read};
use values::OptionKey;

pub const SETTINGS_FILE: &str = "settings.json";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct SavedOptions(Vec<ModelOptions>);

impl SavedOptions {
    pub fn model_options(&self) -> &Vec<ModelOptions> {
        &self.0
    }

    pub fn model_options_mut(&mut self) -> &mut Vec<ModelOptions> {
        &mut self.0
    }

    pub fn set_model_options(&mut self, options: Vec<ModelOptions>) {
        self.0 = options;
    }

    pub fn update_model_option<F>(&mut self, index: usize, mut f: F)
    where
        F: FnMut(&mut ModelOptions),
    {
        f(&mut self.0[index]);
    }

    pub fn update_gen_option<F>(&mut self, model_index: usize, index: usize, f: F)
    where
        F: FnMut(&mut GenOption),
    {
        self.0[model_index].update_option(index, f);
    }

    pub fn gen_option(&mut self, model_index: usize, index: usize) -> &GenOption {
        &self.model_options()[model_index].options()[index]
    }
}

impl SavedOptions {
    pub fn get_model_options_index(&self, model: String) -> Option<usize> {
        self.0.iter().position(|x| x.1 == model)
    }

    pub fn get_create_model_options_index(&mut self, model: String) -> usize {
        let index = self.get_model_options_index(model.clone());
        if let None = index {
            self.0.push(ModelOptions::new(model));
            return self.0.len() - 1;
        }

        if index.unwrap_or(0) > self.0.len() - 1 {
            return 0;
        }

        index.unwrap_or(0)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ModelOptions(Vec<GenOption>, String);

impl ModelOptions {
    pub fn model(&self) -> &str {
        &self.1
    }

    pub fn set_model(&mut self, model: String) {
        self.1 = model;
    }

    pub fn options(&self) -> &Vec<GenOption> {
        &self.0
    }

    pub fn set_options(&mut self, options: Vec<GenOption>) {
        self.0 = options;
    }

    pub fn update_options<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Vec<GenOption>),
    {
        f(&mut self.0);
    }

    pub fn update_option<F>(&mut self, index: usize, mut f: F)
    where
        F: FnMut(&mut GenOption),
    {
        f(&mut self.0[index]);
    }
}

impl SavedOptions {
    pub fn save(&self, path: &str) {
        let path = get_path_settings(path.to_string());
        let writer = File::create(path);

        if let Ok(writer) = writer {
            let _ = serde_json::to_writer(writer, &self);
        }
    }

    pub fn load(path: &str) -> Result<Self, String> {
        let path = get_path_settings(path.to_string());
        let reader = File::open(path);

        if let Ok(mut reader) = reader {
            let mut data = String::new();
            let _ = reader
                .read_to_string(&mut data)
                .map_err(|e| e.to_string())?;

            let de_data = serde_json::from_str(&data);

            return match de_data {
                Ok(x) => Ok(x),
                Err(e) => Err(e.to_string()),
            };
        }

        return Err("Failed to open file".to_string());
    }
}

impl ModelOptions {
    pub fn view<'a>(&'a self, app: &ChatApp, key: Id) -> Element<'a, Message> {
        scrollable(column(self.0.iter().map(|x| {
            let shown = if let Some(y) = &app.main_view.options().get(&key).unwrap().1 {
                if y.clone() == x.key {
                    true
                } else {
                    false
                }
            } else {
                false
            };

            x.view(key, shown)
        })))
        .into()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GenOption {
    pub name: String,
    pub key: OptionKey,
    num_type: NumType,
    pub temp: String,
    pub bool_value: bool,
    pub num_value: Option<(f32, f32)>,
    pub text_value: Option<(String, String)>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum NumType {
    Decimal,
    Whole,
}

impl GenOption {
    fn new(
        name: &str,
        key: OptionKey,
        num_value: Option<(f32, f32)>,
        text_value: Option<(String, String)>,
    ) -> Self {
        Self {
            name: name.to_string(),
            num_type: NumType::Whole,
            temp: num_value.unwrap().0.to_string(),
            key,
            bool_value: false,
            num_value,
            text_value,
        }
    }

    fn _with_type(&mut self, num_type: NumType) {
        self.num_type = num_type;
    }

    pub fn view<'a>(&'a self, key: Id, shown: bool) -> Element<'a, Message> {
        if shown {
            let name = button(text(&self.name).center().size(16))
                .on_press(Message::Option(
                    OptionMessage::ClickedOption(self.key.clone()),
                    key.clone(),
                ))
                .style(style::button::chosen_chat);
            let index = self.key.get_doc_index();
            let doc = container(text(DOCS[index]).center().size(12))
                .padding(5)
                .style(style::container::code);
            let mut widgets: Vec<Element<Message>> = vec![row![
                toggler(self.bool_value)
                    .label("Activated")
                    .on_toggle(move |x| Message::Option(
                        OptionMessage::ChangeOptionBool((x, self.key.clone())),
                        key.clone()
                    ))
                    .width(Length::FillPortion(3)),
                button(
                    text("Reset")
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
                        .width(Length::Fill)
                        .size(16)
                )
                .width(Length::FillPortion(2))
                .style(style::button::not_chosen_chat)
                .on_press(Message::Option(
                    OptionMessage::ResetOption(self.key.clone()),
                    key.clone()
                )),
            ]
            .spacing(10)
            .into()];

            if let Some(x) = self.num_value {
                widgets.push(
                    text_input(&x.1.to_string(), &self.temp)
                        .on_input(move |x| {
                            Message::Option(
                                OptionMessage::ChangeOptionNum((x, self.key.clone())),
                                key.clone(),
                            )
                        })
                        .on_submit(Message::Option(
                            OptionMessage::SubmitOptionNum(self.key.clone()),
                            key.clone(),
                        ))
                        .into(),
                );
            }
            let settings = container(column(widgets));

            container(column![name, doc, settings,])
                .style(style::container::code_darkened)
                .padding(10)
                .into()
        } else {
            button(text(&self.name).center().size(16))
                .on_press(Message::Option(
                    OptionMessage::ClickedOption(self.key.clone()),
                    key.clone(),
                ))
                .style(style::button::not_chosen_chat)
                .into()
        }
    }
}
