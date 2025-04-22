pub mod convert;
pub mod doc;
pub mod values;
use crate::{
    common::Id,
    llm::delete_model,
    style,
    utils::{generate_id, get_path_assets, get_path_settings},
    ChatApp, Message,
};
use doc::DOCS;
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, combo_box, container, row, scrollable, svg, text, text_input, toggler,
    },
    Element, Length, Padding, Task,
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{fs::File, io::Read};

pub const SETTINGS_FILE: &str = "settings.json";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Options(Id, String, Option<OptionKey>);

impl Options {
    pub fn new(model: String) -> Self {
        Self(Id::new(), model, None)
    }

    pub fn model(&self) -> &str {
        &self.1
    }

    pub fn set_model(&mut self, model: String) {
        self.1 = model;
    }

    pub fn id(&self) -> &Id {
        &self.0
    }

    pub fn key(&self) -> &Option<OptionKey> {
        &self.2
    }

    pub fn set_key(&mut self, key: Option<OptionKey>) {
        self.2 = key;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct SavedOptions(pub Vec<ModelOptions>);

#[derive(Debug, Clone)]
pub enum OptionMessage {
    ChangeOptionNum((String, OptionKey)),
    SubmitOptionNum(OptionKey),
    ChangeOptionBool((bool, OptionKey)),
    ClickedOption(OptionKey),
    ResetOption(OptionKey),
    ChangeModel(String),
    DeleteModel,
}

impl OptionMessage {
    pub fn handle<'a>(&'a self, options: Options, app: &'a mut ChatApp) -> Task<Message> {
        match self {
            Self::ChangeOptionBool(x) => {
                let m_index = app
                    .options
                    .get_create_model_options_index(options.model().to_string());
                let index = app.options.0[m_index].get_key_index(x.1.clone());
                app.options.0[m_index].0[index].bool_value = x.0;
                app.options.save(SETTINGS_FILE);
                Task::none()
            }
            Self::ChangeModel(x) => {
                app.main_view.update_option(
                    Options::get_index(app, options.id().clone()),
                    |option| {
                        option.set_model(x.clone());
                    },
                );

                Task::none()
            }
            Self::DeleteModel => {
                let index = Options::get_index(app, options.id().clone());
                let model = app.main_view.options()[index].model().to_string();

                if let Ok(i) = app.logic.models.binary_search(&model) {
                    app.logic.models.remove(i);
                    if let Some(m) = app.logic.models.first() {
                        app.main_view
                            .update_option(index, |option| option.set_model(m.clone()));
                        return Task::perform(
                            delete_model(app.logic.ollama.clone(), model.clone()),
                            move |_| Message::None,
                        );
                    }
                }

                Task::none()
            }
            Self::ChangeOptionNum(x) => {
                let m_index = app
                    .options
                    .get_create_model_options_index(options.model().to_string());
                let index = app.options.0[m_index].get_key_index(x.1.clone());
                app.options.0[m_index].0[index].temp = x.0.clone();
                Task::none()
            }
            Self::SubmitOptionNum(x) => {
                let m_index = app
                    .options
                    .get_create_model_options_index(options.model().to_string());
                let index = app.options.0[m_index].get_key_index(x.clone());
                if let Ok(num) = app.options.0[m_index].0[index].temp.parse::<f32>() {
                    let mut value = app.options.0[m_index].0[index].num_value.unwrap();
                    value.0 = num;
                    app.options.0[m_index].0[index].num_value = Some(value);
                    app.options.save(SETTINGS_FILE);
                } else {
                    app.options.0[m_index].0[index].temp = app.options.0[m_index].0[index]
                        .num_value
                        .unwrap()
                        .0
                        .to_string();
                }
                Task::none()
            }
            Self::ResetOption(x) => {
                let m_index = app
                    .options
                    .get_create_model_options_index(options.model().to_string());
                let index = app.options.0[m_index].get_key_index(x.clone());
                let mut value = app.options.0[m_index].0[index].num_value.unwrap();
                value.0 = value.1;
                app.options.0[m_index].0[index].num_value = Some(value);
                app.options.0[m_index].0[index].temp = value.1.to_string();
                app.options.0[m_index].0[index].bool_value = false;
                app.options.save(SETTINGS_FILE);
                Task::none()
            }
            Self::ClickedOption(x) => {
                let index = Options::get_index(app, options.id().clone());
                if let Some(y) = &app.main_view.options()[index].2 {
                    if x == y {
                        app.main_view.update_option(index, |x| x.set_key(None));
                        return Task::none();
                    }
                }

                app.main_view
                    .update_option(index, |y| y.set_key(Some(x.clone())));
                Task::none()
            }
        }
    }
}

impl SavedOptions {
    pub fn get_model_options_index(&self, model: String) -> Option<usize> {
        for i in 0..self.0.len() {
            if self.0[i].1 == model {
                return Some(i);
            }
        }

        None
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
pub struct ModelOptions(pub Vec<GenOption>, pub String);

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
            let _ = reader.read_to_string(&mut data).unwrap();

            let de_data = serde_json::from_str(&data);

            return match de_data {
                Ok(x) => Ok(x),
                Err(e) => Err(e.to_string()),
            };
        }

        return Err("Failed to open file".to_string());
    }
}

impl Options {
    pub fn get_from_id<'a>(app: &'a ChatApp, id: Id) -> &'a Self {
        app.main_view
            .options()
            .iter()
            .find(|x| x.id() == &id)
            .unwrap()
    }

    pub fn get_index<'a>(app: &'a ChatApp, id: Id) -> usize {
        for i in 0..app.main_view.options().len() {
            if app.main_view.options()[i].id() == &id {
                return i;
            }
        }
        0
    }

    pub fn view<'a>(&'a self, app: &'a ChatApp) -> Element<'a, Message> {
        let index = match app
            .options
            .get_model_options_index(self.model().to_string())
        {
            Some(x) => x,
            None => return text("Failed").into(),
        };
        self.view_with_index(app, index, self.id().clone(), &self.model())
    }

    pub fn view_with_index<'a>(
        &'a self,
        app: &'a ChatApp,
        index: usize,
        id: Id,
        model: &'a str,
    ) -> Element<'a, Message> {
        container(column![
            container(row![
                combo_box(&app.logic.combo_models, model, None, move |x| {
                    Message::Option(OptionMessage::ChangeModel(x), id)
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
                .on_press(Message::Option(OptionMessage::DeleteModel, id))
            ])
            .padding(10),
            container(app.options.0[index].view(self))
                .padding(Padding::default().left(20).right(20).top(5).bottom(5))
        ])
        .into()
    }
}

impl ModelOptions {
    pub fn view<'a>(&'a self, options: &'a Options) -> Element<'a, Message> {
        scrollable(column(self.0.iter().map(|x| {
            let shown = if let Some(y) = &options.2 {
                if y.clone() == x.key {
                    true
                } else {
                    false
                }
            } else {
                false
            };

            x.view(options, shown)
        })))
        .into()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum OptionKey {
    Mirostat,
    MirostatETA,
    MirostatTau,
    CtxWindow,
    NumGQA,
    GPULayers,
    NumThreads,
    RepeatN,
    RepeatPenalty,
    Temperature,
    Seed,
    StopSequence,
    TailFreeZ,
    NumberPredict,
    TopK,
    TopP,
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

    fn with_type(&mut self, num_type: NumType) {
        self.num_type = num_type;
    }

    pub fn view<'a>(&'a self, options: &'a Options, shown: bool) -> Element<'a, Message> {
        if shown {
            let name = button(text(&self.name).center().size(16))
                .on_press(Message::Option(
                    OptionMessage::ClickedOption(self.key.clone()),
                    options.id().clone(),
                ))
                .style(style::button::chosen_chat);
            let index = self.key.get_doc_index();
            let doc = container(text(DOCS[index]).center().size(12))
                .padding(5)
                .style(style::container::code);
            let mut widgets: Vec<Element<Message>> = vec![row![
                toggler(self.bool_value)
                    .label("Activated")
                    .on_toggle(|x| Message::Option(
                        OptionMessage::ChangeOptionBool((x, self.key.clone())),
                        options.id().clone()
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
                    options.id().clone()
                )),
            ]
            .spacing(10)
            .into()];

            if let Some(x) = self.num_value {
                widgets.push(
                    text_input(&x.1.to_string(), &self.temp)
                        .on_input(|x| {
                            Message::Option(
                                OptionMessage::ChangeOptionNum((x, self.key.clone())),
                                options.id().clone(),
                            )
                        })
                        .on_submit(Message::Option(
                            OptionMessage::SubmitOptionNum(self.key.clone()),
                            options.id().clone(),
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
                    options.id().clone(),
                ))
                .style(style::button::not_chosen_chat)
                .into()
        }
    }
}
