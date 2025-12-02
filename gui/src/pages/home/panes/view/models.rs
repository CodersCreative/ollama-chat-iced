use iced::{
    Element, Length, Task,
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container, horizontal_rule, horizontal_space, pick_list, row,
        scrollable::{self, Scrollbar},
        text, text_input,
    },
};
use ochat_types::providers::{Provider, ProviderType, ollama::OllamaModelsInfo};

use crate::{
    Application, DATA, InputMessage, Message,
    font::{BODY_SIZE, HEADER_SIZE, SUB_HEADING_SIZE},
    pages::home::panes::{data::ModelsData, view::HomePaneViewMessage},
    style,
};

#[derive(Debug, Clone)]
pub struct ModelsView {
    pub provider: Option<Provider>,
    pub search: String,
    pub expanded: Vec<String>,
    pub models: ModelsData,
}

impl Default for ModelsView {
    fn default() -> Self {
        Self {
            search: String::default(),
            expanded: Vec::new(),
            models: ModelsData::default(),
            provider: DATA
                .read()
                .unwrap()
                .providers
                .iter()
                .find(|x| x.provider_type == ProviderType::Ollama)
                .map(|x| x.clone()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ModelsViewMessage {
    Search(InputMessage),
    SetModels(ModelsData),
    SetProvider(Provider),
    Expand(String),
    Pull(String, String),
}

impl ModelsViewMessage {
    pub fn handle(self, app: &mut Application, id: u32) -> Task<Message> {
        match self {
            Self::Search(InputMessage::Update(x)) => {
                let view = app.get_models_view(&id).unwrap();

                if x.is_empty() {
                    view.models.0.clear();
                }

                view.search = x;
                Task::none()
            }
            Self::Search(_) => {
                let search = app.get_models_view(&id).unwrap().search.clone();
                Task::future(async move {
                    Message::HomePaneView(HomePaneViewMessage::Models(
                        id,
                        ModelsViewMessage::SetModels(ModelsData::get_ollama(Some(search)).await),
                    ))
                })
            }
            Self::SetModels(x) => {
                app.get_models_view(&id).unwrap().models = x;
                Task::none()
            }
            Self::SetProvider(x) => {
                app.get_models_view(&id).unwrap().provider = Some(x);
                Task::none()
            }
            Self::Expand(x) => {
                let view = app.get_models_view(&id).unwrap();
                if view.expanded.contains(&x) {
                    view.expanded.retain(|y| y != &x);
                } else {
                    view.expanded.push(x);
                }

                Task::none()
            }
            _ => Task::none(),
        }
    }
}

impl ModelsView {
    pub fn view_model<'a>(
        id: u32,
        model: &'a OllamaModelsInfo,
        expanded: bool,
    ) -> Element<'a, Message> {
        let sub_heading = |txt: &'static str| text(txt).size(BODY_SIZE).style(style::text::text);
        let name = text(&model.name)
            .size(HEADER_SIZE)
            .style(style::text::primary);
        let expand = style::svg_button::primary(
            if expanded {
                "arrow_drop_up.svg"
            } else {
                "arrow_drop_down.svg"
            },
            HEADER_SIZE,
        )
        .on_press(Message::HomePaneView(HomePaneViewMessage::Models(
            id,
            ModelsViewMessage::Expand(model.name.clone()),
        )));

        let author = text(&model.author).size(BODY_SIZE);
        let desc = text(&model.description)
            .size(BODY_SIZE)
            .style(style::text::translucent::text);

        let mut col = column![
            row![name, horizontal_space(), expand],
            horizontal_rule(1).style(style::rule::translucent::primary),
            author,
            desc
        ]
        .spacing(10);

        let mut inner_col = column([]).spacing(5).padding(10);

        if expanded {
            if !model.languages.is_empty() {
                inner_col = inner_col.push(sub_heading("Languages"));
                inner_col = inner_col.push(
                    row(model
                        .languages
                        .iter()
                        .map(|x| {
                            container(text(x).size(BODY_SIZE))
                                .style(style::container::side_bar)
                                .padding(5)
                                .into()
                        })
                        .collect::<Vec<Element<'a, Message>>>())
                    .spacing(5),
                )
            }

            if !model.categories.is_empty() {
                inner_col = inner_col.push(sub_heading("Categories"));
                inner_col = inner_col.push(
                    row(model
                        .categories
                        .iter()
                        .map(|x| {
                            container(text(x).size(BODY_SIZE))
                                .style(style::container::side_bar)
                                .padding(5)
                                .into()
                        })
                        .collect::<Vec<Element<'a, Message>>>())
                    .spacing(5),
                )
            }

            if !model.tags.is_empty() {
                inner_col = inner_col.push(sub_heading("Tags"));

                for i in (0..model.tags.len()).filter(|x| x % 2 == 0) {
                    let first = model.tags.get(i).unwrap();
                    let first = button(
                        row![
                            text(first[0].clone())
                                .width(Length::Fill)
                                .align_x(Horizontal::Center)
                                .align_y(Vertical::Center),
                            text(first[1].clone())
                                .width(Length::Fill)
                                .align_x(Horizontal::Center)
                                .align_y(Vertical::Center)
                        ]
                        .spacing(10),
                    )
                    .on_press(Message::HomePaneView(HomePaneViewMessage::Models(
                        id,
                        ModelsViewMessage::Pull(model.name.clone(), first[0].clone()),
                    )))
                    .style(style::button::start)
                    .width(Length::Fill);
                    let second = model.tags.get(i + 1);

                    let second: Element<'a, Message> = match second {
                        Some(second) => button(
                            row![
                                text(second[0].clone())
                                    .width(Length::Fill)
                                    .align_x(Horizontal::Center)
                                    .align_y(Vertical::Center),
                                text(second[1].clone())
                                    .width(Length::Fill)
                                    .align_x(Horizontal::Center)
                                    .align_y(Vertical::Center)
                            ]
                            .spacing(10),
                        )
                        .width(Length::Fill)
                        .style(style::button::start)
                        .on_press(Message::HomePaneView(HomePaneViewMessage::Models(
                            id,
                            ModelsViewMessage::Pull(model.name.clone(), second[0].clone()),
                        )))
                        .into(),
                        None => horizontal_space().into(),
                    };

                    inner_col = inner_col.push(row![first, second].spacing(10));
                }
            }

            col = col.push(container(inner_col).style(style::container::window_title_back))
        };

        container(col).into()
    }

    pub fn view<'a>(&'a self, app: &'a Application, id: u32) -> Element<'a, Message> {
        let provider = pick_list(
            DATA.read()
                .unwrap()
                .providers
                .clone()
                .into_iter()
                .filter(|x| x.provider_type == ProviderType::Ollama)
                .collect::<Vec<Provider>>(),
            self.provider.clone(),
            move |x| {
                Message::HomePaneView(HomePaneViewMessage::Models(
                    id,
                    ModelsViewMessage::SetProvider(x),
                ))
            },
        )
        .style(style::pick_list::main)
        .menu_style(style::menu::main);

        let search = style::svg_input::primary(
            Some(String::from("search.svg")),
            text_input("Search models...", &self.search)
                .on_input(move |x| {
                    Message::HomePaneView(HomePaneViewMessage::Models(
                        id,
                        ModelsViewMessage::Search(InputMessage::Update(x)),
                    ))
                })
                .on_submit(Message::HomePaneView(HomePaneViewMessage::Models(
                    id,
                    ModelsViewMessage::Search(InputMessage::Submit),
                ))),
            SUB_HEADING_SIZE,
        );

        let models = scrollable::Scrollable::new(
            column(
                if self.search.is_empty() || self.models.0.is_empty() {
                    &app.cache.home_shared.models.0
                } else {
                    &self.models.0
                }
                .iter()
                .map(|x| Self::view_model(id.clone(), x, self.expanded.contains(&x.name))),
            )
            .spacing(10),
        )
        .direction(scrollable::Direction::Vertical(Scrollbar::new()))
        .width(Length::Fill)
        .height(Length::Fill);

        container(column![row![search, provider,], models].spacing(10)).into()
    }
}
