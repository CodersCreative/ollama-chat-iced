use crate::{
    Application, DATA, InputMessage, Message,
    font::{BODY_SIZE, HEADER_SIZE, SUB_HEADING_SIZE},
    pages::home::panes::{data::ModelsData, view::HomePaneViewMessage},
    style,
    subscriptions::SubMessage,
    utils::{get_path_assets, print_data_size},
};
use iced::{
    Element, Length, Task, Theme,
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container, grid, markdown, pick_list, row, rule,
        scrollable::{self, Scrollbar},
        space, svg, text, text_input,
    },
};
use ochat_types::{
    providers::{
        Provider, ProviderType,
        hf::{HFModel, HFModelDetails, HFModelVariant},
        ollama::OllamaModelsInfo,
    },
    settings::SettingsProvider,
};
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone)]
pub struct ModelsView {
    pub provider: Option<Provider>,
    pub page: Page,
    pub search: String,
    pub ollama_expanded: Vec<String>,
    pub hf_expanded: HashMap<String, HFViewModelDetails>,
    pub models: ModelsData,
}

#[derive(Debug)]
pub struct HFViewModelDetails {
    pub description: markdown::Content,
    pub base: HFModelDetails,
}

impl From<HFModelDetails> for HFViewModelDetails {
    fn from(value: HFModelDetails) -> Self {
        Self {
            description: markdown::Content::parse(&value.description),

            base: value,
        }
    }
}

impl Clone for HFViewModelDetails {
    fn clone(&self) -> Self {
        Self {
            description: markdown::Content::parse(&self.base.description),
            base: self.base.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Page {
    Ollama,
    HF,
}

impl Page {
    pub const ALL: [Page; 2] = [Page::Ollama, Page::HF];
}

impl Display for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Page::Ollama => "Ollama",
                Page::HF => "Huggingface",
            }
        )
    }
}

impl Default for ModelsView {
    fn default() -> Self {
        Self {
            search: String::default(),
            page: Page::Ollama,
            ollama_expanded: Vec::new(),
            hf_expanded: HashMap::new(),
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
    ChangePage(Page),
    OllamaExpand(String),
    HFExpand(String),
    SetHFExpand(String, HFModelDetails),
}

impl ModelsViewMessage {
    pub fn handle(self, app: &mut Application, id: u32) -> Task<Message> {
        match self {
            Self::Search(InputMessage::Update(x)) => {
                let view = app.get_models_view(&id).unwrap();

                if x.is_empty() {
                    view.models.ollama.clear();
                    view.models.hf.clear();
                }

                view.search = x;
                Task::none()
            }
            Self::Search(_) => {
                let search = app.get_models_view(&id).unwrap().search.clone();
                Task::future(async move {
                    match ModelsData::get(Some(search)).await {
                        Ok(x) => Message::HomePaneView(HomePaneViewMessage::Models(
                            id,
                            ModelsViewMessage::SetModels(x),
                        )),
                        Err(e) => Message::Err(e),
                    }
                })
            }
            Self::ChangePage(x) => {
                app.get_models_view(&id).unwrap().page = x;
                Task::none()
            }
            Self::SetModels(x) => {
                app.get_models_view(&id).unwrap().models = x;
                Task::none()
            }
            Self::SetProvider(x) => {
                app.get_models_view(&id).unwrap().provider = Some(x);
                Task::none()
            }
            Self::OllamaExpand(x) => {
                let view = app.get_models_view(&id).unwrap();
                if view.ollama_expanded.contains(&x) {
                    view.ollama_expanded.retain(|y| y != &x);
                } else {
                    view.ollama_expanded.push(x);
                }

                Task::none()
            }
            Self::SetHFExpand(x, details) => {
                app.get_models_view(&id)
                    .unwrap()
                    .hf_expanded
                    .insert(x, details.into());
                Task::none()
            }
            Self::HFExpand(x) => {
                let view = app.get_models_view(&id).unwrap();

                if view.hf_expanded.contains_key(&x) {
                    view.hf_expanded.retain(|y, _| y != &x);
                    Task::none()
                } else {
                    Task::future(async move {
                        let req = DATA.read().unwrap().to_request();
                        match req
                            .make_request(
                                &format!("provider/hf/model/{}", x),
                                &(),
                                crate::data::RequestType::Get,
                            )
                            .await
                        {
                            Ok(y) => Message::HomePaneView(HomePaneViewMessage::Models(
                                id,
                                ModelsViewMessage::SetHFExpand(x.clone(), y),
                            )),
                            Err(e) => Message::Err(e),
                        }
                    })
                }
            }
        }
    }
}

impl ModelsView {
    pub fn view_ollama_model<'a>(
        id: u32,
        model: &'a OllamaModelsInfo,
        provider: String,
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
            ModelsViewMessage::OllamaExpand(model.name.clone()),
        )));

        let author = text(&model.author).size(BODY_SIZE);
        let desc = text(&model.description)
            .size(BODY_SIZE)
            .style(style::text::translucent::text);

        let mut col = column![
            row![name, space::horizontal(), expand],
            rule::horizontal(1).style(style::rule::translucent::primary),
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

                inner_col = inner_col.push(
                    grid(model.tags.iter().map(|x| {
                        let mut btn = button(
                            row![
                                text(x[0].clone())
                                    .width(Length::Fill)
                                    .align_x(Horizontal::Center)
                                    .align_y(Vertical::Center),
                                text(x[1].clone())
                                    .width(Length::Fill)
                                    .align_x(Horizontal::Center)
                                    .align_y(Vertical::Center)
                            ]
                            .spacing(10),
                        )
                        .style(style::button::start)
                        .width(Length::Fill);

                        if !DATA.read().unwrap().models.contains(&SettingsProvider {
                            provider: provider.clone(),
                            model: format!("{}:{}", &model.name, &x[0]),
                        }) {
                            btn = btn.on_press(Message::Subscription(SubMessage::OllamaPull(
                                model.clone(),
                                SettingsProvider {
                                    provider: provider.clone(),
                                    model: format!("{}:{}", &model.name, &x[0]),
                                },
                            )));
                        }
                        btn.into()
                    }))
                    .spacing(10)
                    .height(Length::Shrink),
                );
            }

            col = col.push(container(inner_col).style(style::container::window_title_back))
        };

        container(col).into()
    }

    pub fn view_hf_model<'a>(
        id: u32,
        model: &'a HFModel,
        theme: &Theme,
        expanded: Option<&'a HFViewModelDetails>,
    ) -> Element<'a, Message> {
        let sub_heading = |txt: &'static str| text(txt).size(BODY_SIZE).style(style::text::text);
        let stat = |icon: &'static str, txt: String| {
            row![
                svg(svg::Handle::from_path(get_path_assets(icon)))
                    .style(style::svg::text)
                    .width(Length::Fixed(BODY_SIZE as f32)),
                text(txt).size(BODY_SIZE),
            ]
            .spacing(5)
            .align_y(Vertical::Center)
        };
        let name = text(model.id.split_once('/').unwrap().1)
            .size(HEADER_SIZE)
            .style(style::text::primary);
        let expand = style::svg_button::primary(
            if expanded.is_some() {
                "arrow_drop_up.svg"
            } else {
                "arrow_drop_down.svg"
            },
            HEADER_SIZE,
        )
        .on_press(Message::HomePaneView(HomePaneViewMessage::Models(
            id,
            ModelsViewMessage::HFExpand(model.id.clone()),
        )));

        let author = text(model.id.split_once('/').unwrap().0).size(BODY_SIZE);

        let mut stats = row![
            stat("downloads.svg", model.downloads.to_string()),
            stat("thumbs_up.svg", model.likes.to_string()),
            stat("schedule.svg", model.last_modified.0.to_string()),
        ]
        .spacing(10)
        .align_y(Vertical::Center);

        let mut col = column![
            row![name, space::horizontal(), expand],
            rule::horizontal(1).style(style::rule::translucent::primary),
            author,
        ]
        .spacing(10);

        let mut inner_col = column([]).spacing(10).padding(10);

        if let Some(expanded) = expanded {
            if let Some(arch) = &expanded.base.architecture {
                stats = stats.push(stat("arch.svg", arch.clone()))
            };

            stats = stats.push(stat(
                "params.svg",
                print_data_size(&expanded.base.parameters),
            ));

            col = col.push(container(stats));

            inner_col = inner_col.push(sub_heading("Tags"));

            let tags: Element<'a, Message> = if expanded.base.variants.0.is_empty() {
                text("No compatible files have been found.")
                    .style(style::text::text)
                    .size(BODY_SIZE + 2)
                    .into()
            } else {
                let mut variants: Vec<(&u64, &Vec<HFModelVariant>)> =
                    expanded.base.variants.0.iter().collect();
                variants.sort_by(|a, b| a.0.cmp(b.0));
                column(variants.iter().map(|(k, variants)| {
                    container(
                        row![
                            text(format!("{} bit", k))
                                .style(style::text::primary)
                                .size(SUB_HEADING_SIZE)
                                .width(100),
                            grid(variants.iter().map(|variant| {
                                button(
                                    row![
                                        text(variant.variant().unwrap_or_default())
                                            .style(style::text::text)
                                            .size(BODY_SIZE),
                                        text(print_data_size(&variant.size.clone().unwrap_or(0)))
                                            .style(style::text::translucent::text)
                                            .size(BODY_SIZE)
                                    ]
                                    .spacing(5)
                                    .padding(5)
                                    .align_y(Vertical::Center),
                                )
                                .style(style::button::start)
                                .on_press(Message::Subscription(SubMessage::HFPull(
                                    model.clone(),
                                    variant.name.clone(),
                                )))
                                .into()
                            }))
                            .height(Length::Shrink)
                            .spacing(10),
                        ]
                        .align_y(Vertical::Center)
                        .spacing(20),
                    )
                    .padding(20)
                    .style(style::container::code)
                    .into()
                }))
                .spacing(5)
                .into()
            };

            inner_col = inner_col.push(tags);

            inner_col = inner_col.push(sub_heading("Readme"));
            inner_col = inner_col.push(markdown::view_with(
                expanded.description.items(),
                style::markdown::main(theme),
                &style::markdown::CustomViewer,
            ));

            col = col.push(container(inner_col).style(style::container::window_title_back))
        } else {
            col = col.push(container(stats));
        };

        container(col).into()
    }

    pub fn view<'a>(&'a self, app: &'a Application, id: u32) -> Element<'a, Message> {
        let page = pick_list(Page::ALL, Some(self.page.clone()), move |x| {
            Message::HomePaneView(HomePaneViewMessage::Models(
                id,
                ModelsViewMessage::ChangePage(x),
            ))
        })
        .style(style::pick_list::main)
        .menu_style(style::menu::main);

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

        let p = self
            .provider
            .clone()
            .map(|x| x.id.key().to_string())
            .unwrap_or_default();

        let models = scrollable::Scrollable::new(match self.page {
            Page::Ollama => column(
                if self.search.is_empty() || self.models.ollama.is_empty() {
                    &app.cache.home_shared.models.ollama
                } else {
                    &self.models.ollama
                }
                .iter()
                .map(|x| {
                    Self::view_ollama_model(
                        id.clone(),
                        x,
                        p.clone(),
                        self.ollama_expanded.contains(&x.name),
                    )
                }),
            )
            .spacing(10),
            Page::HF => column(
                if self.search.is_empty() || self.models.hf.is_empty() {
                    &app.cache.home_shared.models.hf
                } else {
                    &self.models.hf
                }
                .iter()
                .map(|x| {
                    Self::view_hf_model(id.clone(), x, &app.theme(), self.hf_expanded.get(&x.id))
                }),
            )
            .spacing(10),
        })
        .direction(scrollable::Direction::Vertical(Scrollbar::new()))
        .width(Length::Fill)
        .height(Length::Fill);

        let header = if self.page == Page::Ollama {
            row![provider, search, page]
        } else {
            row![search, page,]
        };

        container(column![header, models].spacing(10)).into()
    }
}
