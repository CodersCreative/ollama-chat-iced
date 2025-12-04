use crate::{
    Application, DATA, InputMessage, Message,
    font::{BODY_SIZE, HEADER_SIZE, SUB_HEADING_SIZE},
    pages::home::panes::{
        data::{OptionData, OptionsData},
        view::HomePaneViewMessage,
    },
    style,
};
use iced::{
    Element, Length, Task,
    alignment::Vertical,
    widget::{
        column, container, horizontal_rule, horizontal_space, row,
        scrollable::{self, Scrollbar},
        text, text_input,
    },
};
use ochat_types::options::{GenOptions, GenOptionsData};
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
    UpdateField(String, String),
    SaveOption(String),
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
            _ => Task::none(),
        }
    }
}

impl OptionsView {
    pub fn view_option<'a>(
        id: u32,
        option: &'a OptionData,
        expanded: bool,
        edit_data: Option<&'a OptionData>,
    ) -> Element<'a, Message> {
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

        let mut col = if let Some(edit_data) = edit_data {
            let sub_heading =
                |txt: &'static str| text(txt).size(BODY_SIZE).style(style::text::primary);

            let name = text_input("Enter a title...", &edit_data.option.name)
                .size(HEADER_SIZE)
                .style(style::text_input::input);

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

            let mut col = column![
                row![delete, name, horizontal_space(), edit, save].align_y(Vertical::Center),
                horizontal_rule(1).style(style::rule::translucent::primary),
                sub_heading("Options"),
            ]
            .spacing(10);

            col
        } else if expanded {
            let mut col = column![
                row![name, horizontal_space(), expand].align_y(Vertical::Center),
                horizontal_rule(1).style(style::rule::translucent::primary),
            ]
            .spacing(10);

            col
        } else {
            column![
                row![name, horizontal_space(), edit, expand].align_y(Vertical::Center),
                horizontal_rule(1).style(style::rule::translucent::primary),
            ]
            .spacing(10)
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
