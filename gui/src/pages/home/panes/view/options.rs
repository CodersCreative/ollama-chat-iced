use crate::{
    Application, DATA, InputMessage, Message,
    font::{BODY_SIZE, HEADER_SIZE, SUB_HEADING_SIZE},
    pages::home::panes::{
        data::{OptionData, OptionRelationshipData, OptionsData},
        view::HomePaneViewMessage,
    },
    style,
};
use iced::{
    Element, Length, Task,
    alignment::Vertical,
    widget::{
        column, container, pick_list, row, rule,
        scrollable::{self, Scrollbar},
        space, text, text_input, toggler,
    },
};
use ochat_types::{
    options::{GenOption, GenOptions, GenOptionsData},
    settings::SettingsProvider,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct OptionsView {
    pub search: String,
    pub expanded: Vec<String>,
    pub editing: HashMap<String, OptionData>,
    pub options: OptionsData,
}

#[derive(Debug, Clone)]
pub enum OptionsViewMessage {
    Search(InputMessage),
    UpdateName(String, String),
    UpdateField(String, usize, String),
    UpdateFieldActivated(String, usize, bool),
    ResetField(String, usize),
    SaveOption(String),
    UpdateRelationshipModel(String, usize, SettingsProvider),
    AddRelationship(String),
    SetOptions(OptionsData),
    Expand(String),
    Edit(String),
    Delete(String),
    Add,
    AddOption(OptionData),
}

impl OptionsViewMessage {
    pub fn handle(self, app: &mut Application, id: u32) -> Task<Message> {
        match self {
            Self::Search(InputMessage::Update(x)) => {
                let view = app.get_options_view(&id).unwrap();

                if x.is_empty() {
                    view.options.0.clear();
                }

                view.search = x;
                Task::none()
            }
            Self::Search(_) => {
                let search = app.get_options_view(&id).unwrap().search.clone();
                Task::future(async move {
                    Message::HomePaneView(HomePaneViewMessage::Options(
                        id,
                        OptionsViewMessage::SetOptions(
                            OptionsData::get_gen_models(Some(search)).await,
                        ),
                    ))
                })
            }
            Self::AddRelationship(option_id) => {
                app.get_options_view(&id)
                    .unwrap()
                    .editing
                    .get_mut(&option_id)
                    .unwrap()
                    .models
                    .push(OptionRelationshipData {
                        model: None,
                        option: option_id.clone(),
                        id: None,
                    });
                Task::none()
            }
            Self::UpdateRelationshipModel(option_id, index, model) => {
                app.get_options_view(&id)
                    .unwrap()
                    .editing
                    .get_mut(&option_id)
                    .unwrap()
                    .models
                    .get_mut(index)
                    .unwrap()
                    .model = Some(model);
                Task::none()
            }
            Self::UpdateName(option_id, name) => {
                app.get_options_view(&id)
                    .unwrap()
                    .editing
                    .get_mut(&option_id)
                    .unwrap()
                    .option
                    .name = name;
                Task::none()
            }
            Self::UpdateFieldActivated(option_id, index, value) => {
                app.get_options_view(&id)
                    .unwrap()
                    .editing
                    .get_mut(&option_id)
                    .unwrap()
                    .option
                    .data
                    .get_mut(index)
                    .unwrap()
                    .activated = value;

                Task::none()
            }
            Self::ResetField(option_id, index) => {
                app.get_options_view(&id)
                    .unwrap()
                    .editing
                    .get_mut(&option_id)
                    .unwrap()
                    .option
                    .data
                    .get_mut(index)
                    .unwrap()
                    .reset();

                Task::none()
            }
            Self::UpdateField(option_id, index, value) => {
                let field = app
                    .get_options_view(&id)
                    .unwrap()
                    .editing
                    .get_mut(&option_id)
                    .unwrap()
                    .option
                    .data
                    .get_mut(index)
                    .unwrap();

                let value = match field.value {
                    ochat_types::options::GenOptionValue::Float(_) => {
                        ochat_types::options::GenOptionValue::Float(match value.parse() {
                            Ok(x) => x,
                            _ => return Task::none(),
                        })
                    }
                    ochat_types::options::GenOptionValue::Int(_) => {
                        ochat_types::options::GenOptionValue::Int(match value.parse() {
                            Ok(x) => x,
                            _ => return Task::none(),
                        })
                    }
                    ochat_types::options::GenOptionValue::Text(_) => {
                        ochat_types::options::GenOptionValue::Text(value)
                    }
                };

                field.value = value;
                Task::none()
            }
            Self::SetOptions(x) => {
                app.get_options_view(&id).unwrap().options = x;
                Task::none()
            }
            Self::SaveOption(option_id) => {
                let mut option = app
                    .cache
                    .home_shared
                    .options
                    .0
                    .iter()
                    .find(|x| x.option.id.key().to_string() == option_id)
                    .unwrap()
                    .option
                    .clone();

                let edit = app
                    .get_options_view(&id)
                    .unwrap()
                    .editing
                    .get(&option_id)
                    .unwrap()
                    .clone();

                option.name = edit.option.name;
                option.data = edit.option.data;

                app.cache
                    .home_shared
                    .options
                    .0
                    .iter_mut()
                    .filter(|x| x.option.id.key().to_string() == option_id)
                    .for_each(|x| x.option = option.clone());

                app.view_data
                    .home
                    .options
                    .iter_mut()
                    .filter(|x| !x.1.options.0.is_empty())
                    .for_each(|x| {
                        x.1.options
                            .0
                            .iter_mut()
                            .filter(|x| x.option.id.key().to_string() == option_id)
                            .for_each(|x| x.option = option.clone())
                    });

                Task::future(async move {
                    let req = DATA.read().unwrap().to_request();
                    let option: GenOptionsData = option.into();

                    let _ = req
                        .make_request::<GenOptions, GenOptionsData>(
                            &format!("option/{}", option_id),
                            &option,
                            crate::data::RequestType::Put,
                        )
                        .await;
                    Message::None
                })
            }
            Self::Expand(x) => {
                let view = app.get_options_view(&id).unwrap();
                if view.expanded.contains(&x) {
                    view.expanded.retain(|y| y != &x);
                } else {
                    view.expanded.push(x);
                }

                Task::none()
            }
            Self::Edit(x) => {
                let option = app
                    .cache
                    .home_shared
                    .options
                    .0
                    .iter()
                    .find(|y| &y.option.id.key().to_string() == &x)
                    .unwrap()
                    .clone();

                let view = app.get_options_view(&id).unwrap();

                if view.editing.contains_key(&x) {
                    view.editing.remove(&x);
                } else {
                    view.editing.insert(x, option);
                }

                Task::none()
            }
            Self::AddOption(x) => {
                app.cache.home_shared.options.0.push(x);
                Task::none()
            }
            Self::Add => Task::future(async move {
                let req = DATA.read().unwrap().to_request();

                match req
                    .make_request::<GenOptions, GenOptionsData>(
                        "option/",
                        &GenOptionsData {
                            name: String::from("New Options"),
                            ..Default::default()
                        },
                        crate::data::RequestType::Post,
                    )
                    .await
                {
                    Ok(x) => Message::HomePaneView(HomePaneViewMessage::Options(
                        id,
                        OptionsViewMessage::AddOption(OptionData {
                            option: x,
                            models: Vec::new(),
                        }),
                    )),
                    _ => Message::None,
                }
            }),
            Self::Delete(x) => {
                app.cache
                    .home_shared
                    .options
                    .0
                    .retain(|y| y.option.id.key().to_string() != x);

                Task::future(async move {
                    let req = DATA.read().unwrap().to_request();

                    let _ = req
                        .make_request::<GenOptions, ()>(
                            &format!("option/{}", x),
                            &(),
                            crate::data::RequestType::Delete,
                        )
                        .await;

                    Message::None
                })
            }
        }
    }
}

impl OptionsView {
    pub fn view_gen_option<'a>(
        id: u32,
        option_id: String,
        index: usize,
        value: &'a GenOption,
        editing: bool,
    ) -> Element<'a, Message> {
        if editing {
            let name = text(value.key.name())
                .size(SUB_HEADING_SIZE)
                .style(style::text::primary);

            let desc = text(value.key.desc())
                .size(BODY_SIZE)
                .style(style::text::translucent::text);

            let reset = style::svg_button::danger("restart.svg", SUB_HEADING_SIZE).on_press(
                Message::HomePaneView(HomePaneViewMessage::Options(
                    id,
                    OptionsViewMessage::ResetField(option_id.clone(), index),
                )),
            );

            let id2 = option_id.clone();

            let activated = toggler(value.activated)
                .size(SUB_HEADING_SIZE)
                .on_toggle(move |x| {
                    Message::HomePaneView(HomePaneViewMessage::Options(
                        id,
                        OptionsViewMessage::UpdateFieldActivated(id2.clone(), index, x),
                    ))
                });

            let val = text_input("Enter a value...", &value.value.to_string())
                .size(SUB_HEADING_SIZE)
                .style(style::text_input::input)
                .on_input(move |x| {
                    Message::HomePaneView(HomePaneViewMessage::Options(
                        id,
                        OptionsViewMessage::UpdateField(option_id.clone(), index, x),
                    ))
                });

            container(
                column![
                    row![
                        name.width(Length::Fixed(175.0)),
                        space::horizontal(),
                        reset,
                        activated,
                        val
                    ]
                    .width(Length::Fill)
                    .spacing(20)
                    .align_y(Vertical::Center),
                    desc
                ]
                .spacing(10),
            )
            .padding(20)
            .style(style::container::chat_back)
            .into()
        } else {
            let name = text(value.key.name())
                .size(SUB_HEADING_SIZE)
                .style(style::text::primary);

            let desc = text(value.key.desc())
                .size(BODY_SIZE)
                .style(style::text::translucent::text);

            let val = text(value.value.to_string())
                .size(SUB_HEADING_SIZE)
                .style(style::text::text);

            container(
                column![
                    row![name, space::horizontal(), val].align_y(Vertical::Center),
                    desc
                ]
                .spacing(10),
            )
            .padding(20)
            .style(style::container::chat_back)
            .into()
        }
    }

    pub fn view_option<'a>(
        id: u32,
        option: &'a OptionData,
        expanded: bool,
        edit_data: Option<&'a OptionData>,
    ) -> Element<'a, Message> {
        let sub_heading = |txt: &'static str| text(txt).size(BODY_SIZE).style(style::text::primary);
        let edit = style::svg_button::primary(
            if edit_data.is_some() {
                "close.svg"
            } else {
                "edit.svg"
            },
            HEADER_SIZE,
        )
        .on_press(Message::HomePaneView(HomePaneViewMessage::Options(
            id,
            OptionsViewMessage::Edit(option.option.id.key().to_string()),
        )));

        let expand = style::svg_button::primary(
            if expanded {
                "arrow_drop_up.svg"
            } else {
                "arrow_drop_down.svg"
            },
            HEADER_SIZE,
        )
        .on_press(Message::HomePaneView(HomePaneViewMessage::Options(
            id,
            OptionsViewMessage::Expand(option.option.id.key().to_string()),
        )));

        let name = text(&option.option.name)
            .size(HEADER_SIZE)
            .style(style::text::primary);

        let col = if let Some(edit_data) = edit_data {
            let name = text_input("Enter a title...", &edit_data.option.name)
                .size(HEADER_SIZE)
                .style(style::text_input::input)
                .on_input(move |x| {
                    Message::HomePaneView(HomePaneViewMessage::Options(
                        id,
                        OptionsViewMessage::UpdateName(option.option.id.key().to_string(), x),
                    ))
                });

            let delete = style::svg_button::danger("delete.svg", HEADER_SIZE).on_press(
                Message::HomePaneView(HomePaneViewMessage::Options(
                    id,
                    OptionsViewMessage::Delete(option.option.id.key().to_string()),
                )),
            );

            let save = style::svg_button::primary("save.svg", HEADER_SIZE).on_press(
                Message::HomePaneView(HomePaneViewMessage::Options(
                    id,
                    OptionsViewMessage::SaveOption(option.option.id.key().to_string()),
                )),
            );

            column![
                row![delete, name, space::horizontal(), edit, save].align_y(Vertical::Center),
                rule::horizontal(1).style(style::rule::translucent::primary),
                sub_heading("Models"),
                row![
                    {
                        let models = DATA.read().unwrap().models.clone();
                        scrollable::Scrollable::new(
                            row(option.models.iter().enumerate().map(|(i, x)| {
                                pick_list(models.clone(), x.model.clone(), move |x| {
                                    Message::HomePaneView(HomePaneViewMessage::Options(
                                        id,
                                        OptionsViewMessage::UpdateRelationshipModel(
                                            option.option.id.key().to_string(),
                                            i,
                                            x,
                                        ),
                                    ))
                                })
                                .style(style::pick_list::main)
                                .menu_style(style::menu::main)
                                .into()
                            }))
                            .spacing(10),
                        )
                        .direction(scrollable::Direction::Horizontal(Scrollbar::new()))
                        .width(Length::Fill)
                    },
                    space::horizontal(),
                    style::svg_button::primary("add.svg", BODY_SIZE).on_press(
                        Message::HomePaneView(HomePaneViewMessage::Options(
                            id,
                            OptionsViewMessage::AddRelationship(option.option.id.key().to_string()),
                        )),
                    )
                ],
                sub_heading("Options"),
                column(edit_data.option.data.iter().enumerate().map(|(i, x)| {
                    Self::view_gen_option(
                        id.clone(),
                        option.option.id.key().to_string(),
                        i,
                        x,
                        true,
                    )
                }),)
                .spacing(5)
            ]
            .spacing(10)
        } else if expanded {
            let mut col = column![
                row![name, space::horizontal(), edit, expand].align_y(Vertical::Center),
                rule::horizontal(1).style(style::rule::translucent::primary),
            ]
            .spacing(10);

            if !option.models.is_empty() {
                col = col.push(sub_heading("Models"));
                col = col.push(
                    scrollable::Scrollable::new(
                        row(option.models.iter().map(|x| {
                            text(match x.model.clone() {
                                Some(x) => x.model,
                                _ => "New".to_string(),
                            })
                            .size(BODY_SIZE)
                            .into()
                        }))
                        .spacing(10),
                    )
                    .direction(scrollable::Direction::Horizontal(Scrollbar::new()))
                    .width(Length::Fill),
                );
            }

            col = col.push(sub_heading("Options"));
            col = col.push(
                column(option.option.data.iter().enumerate().map(|(i, x)| {
                    Self::view_gen_option(
                        id.clone(),
                        option.option.id.key().to_string(),
                        i,
                        x,
                        false,
                    )
                }))
                .spacing(5),
            );

            col
        } else {
            let mut col = column![
                row![name, space::horizontal(), edit, expand].align_y(Vertical::Center),
                rule::horizontal(1).style(style::rule::translucent::primary),
            ]
            .spacing(10);

            if !option.models.is_empty() {
                col = col.push(sub_heading("Models"));
                col = col.push(
                    scrollable::Scrollable::new(
                        row(option.models.iter().map(|x| {
                            text(match x.model.clone() {
                                Some(x) => x.model,
                                _ => "New".to_string(),
                            })
                            .size(BODY_SIZE)
                            .into()
                        }))
                        .spacing(10),
                    )
                    .direction(scrollable::Direction::Horizontal(Scrollbar::new()))
                    .width(Length::Fill),
                );
            }

            col
        };

        container(col).into()
    }

    pub fn view<'a>(&'a self, app: &'a Application, id: u32) -> Element<'a, Message> {
        let search = style::svg_input::primary(
            Some(String::from("search.svg")),
            text_input("Search prompts...", &self.search)
                .on_input(move |x| {
                    Message::HomePaneView(HomePaneViewMessage::Options(
                        id,
                        OptionsViewMessage::Search(InputMessage::Update(x)),
                    ))
                })
                .on_submit(Message::HomePaneView(HomePaneViewMessage::Options(
                    id,
                    OptionsViewMessage::Search(InputMessage::Submit),
                ))),
            SUB_HEADING_SIZE,
        );

        let add = style::svg_button::primary("add.svg", SUB_HEADING_SIZE).on_press(
            Message::HomePaneView(HomePaneViewMessage::Options(id, OptionsViewMessage::Add)),
        );

        let options = scrollable::Scrollable::new(
            column(
                if self.search.is_empty() || self.options.0.is_empty() {
                    &app.cache.home_shared.options.0
                } else {
                    &self.options.0
                }
                .iter()
                .map(|x| {
                    Self::view_option(
                        id.clone(),
                        x,
                        self.expanded.contains(&x.option.id.key().to_string()),
                        self.editing.get(&x.option.id.key().to_string()),
                    )
                }),
            )
            .spacing(10),
        )
        .direction(scrollable::Direction::Vertical(Scrollbar::new()))
        .width(Length::Fill)
        .height(Length::Fill);

        container(column![row![search, add], options].spacing(10)).into()
    }
}
