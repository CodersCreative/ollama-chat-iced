use iced::{
    Element, Length, Padding, Task, Theme,
    alignment::Vertical,
    border, padding,
    widget::{
        Scrollable, button, center, checkbox, column, container, float, grid, hover, keyed_column,
        pick_list, right_center, row, rule,
        scrollable::{Direction, Scrollbar},
        space, stack, text, text_input,
    },
};
use ochat_types::{
    providers::{Provider, ProviderData, ProviderDataBuilder, ProviderType},
    settings::{SettingsData, SettingsProvider, SettingsProviderBuilder},
    surreal::RecordId,
};
use serde_json::Value;

use crate::{
    Application, DATA, InputMessage, Message,
    data::{Data, RequestType},
    font::{BODY_SIZE, HEADER_SIZE, SUB_HEADING_SIZE},
    pages::home::panes::view::HomePaneViewMessage,
    style::{self},
};

#[derive(Debug, Clone)]
pub struct SettingsView {
    pub instance_url: String,
    pub provider_inputs: Vec<ProviderData>,
}
impl Default for SettingsView {
    fn default() -> Self {
        Self {
            instance_url: String::from("http://localhost:1212"),
            provider_inputs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SettingsViewMessage {
    UpdateProviderName(usize, String),
    UpdateProviderUrl(usize, String),
    UpdateProviderType(usize, ProviderType),
    UpdateProviderKey(usize, String),
    UpdatePreviewModel(SettingsProvider),
    UpdateDefaultModel(SettingsProvider),
    InstanceUrl(InputMessage),
    UpdateUsePanes(bool),
    UpdateTheme(Theme),
    DeleteProvider(RecordId),
    RemoveProviderInput(usize),
    AddProvider(usize),
    AddProviderInput,
}

impl SettingsViewMessage {
    pub fn handle(self, app: &mut Application, id: u32) -> Task<Message> {
        macro_rules! UpdateProviderInputProperty {
            ($index:expr, $prop:ident) => {{
                app.get_settings_view(&id)
                    .unwrap()
                    .provider_inputs
                    .get_mut($index)
                    .unwrap()
                    .$prop = $prop;
                Task::none()
            }};
        }

        macro_rules! UpdateModel {
            ($model:expr, $prop:ident) => {{
                app.cache.settings.$prop = Some($model);
                Task::future(save_settings(app.cache.settings.clone()))
            }};
        }

        match self {
            Self::AddProviderInput => {
                app.get_settings_view(&id).unwrap().provider_inputs.push(
                    ProviderDataBuilder::default()
                        .name(String::new())
                        .url(String::new())
                        .api_key(String::new())
                        .provider_type(ProviderType::Ollama)
                        .build()
                        .unwrap(),
                );
                Task::none()
            }
            Self::AddProvider(index) => {
                let input = app
                    .get_settings_view(&id)
                    .unwrap()
                    .provider_inputs
                    .remove(index);
                Task::future(async move {
                    let req = DATA.read().unwrap().to_request();
                    if let Ok(Some(provider)) = req
                        .make_request::<Option<Provider>, ProviderData>(
                            "provider/",
                            &input,
                            RequestType::Post,
                        )
                        .await
                    {
                        let provider_models: Result<Vec<Value>, String> = req
                            .make_request(
                                &format!("provider/{}/model/all/", provider.id.key()),
                                &(),
                                RequestType::Get,
                            )
                            .await;

                        if let Ok(provider_models) = provider_models {
                            let mut models = Vec::new();

                            for model in provider_models {
                                models.push(
                                    SettingsProviderBuilder::default()
                                        .provider(provider.id.key().to_string())
                                        .model(model["id"].as_str().unwrap().to_string())
                                        .build()
                                        .unwrap(),
                                );
                            }
                            DATA.write().unwrap().models.append(&mut models);
                        }
                        DATA.write().unwrap().providers.push(provider);
                    }

                    Message::None
                })
            }
            Self::DeleteProvider(id) => {
                DATA.write().unwrap().providers.retain(|x| x.id != id);
                DATA.write()
                    .unwrap()
                    .models
                    .retain(|x| x.provider != id.key().to_string());

                Task::future(async move {
                    let req = DATA.read().unwrap().to_request();
                    let _: Value = req
                        .make_request(
                            &format!("provider/{}", id.key().to_string()),
                            &(),
                            RequestType::Delete,
                        )
                        .await
                        .unwrap_or_default();
                    Message::None
                })
            }
            Self::UpdateProviderName(index, name) => UpdateProviderInputProperty!(index, name),
            Self::UpdateProviderUrl(index, url) => UpdateProviderInputProperty!(index, url),
            Self::UpdateProviderType(index, provider_type) => {
                UpdateProviderInputProperty!(index, provider_type)
            }
            Self::UpdateProviderKey(index, api_key) => UpdateProviderInputProperty!(index, api_key),
            Self::UpdatePreviewModel(model) => UpdateModel!(model, previews_provider),
            Self::UpdateDefaultModel(model) => {
                app.cache.client_settings.default_provider = Some(model);
                app.cache.client_settings.save();
                Task::none()
            }
            Self::InstanceUrl(InputMessage::Update(url)) => {
                app.get_settings_view(&id).unwrap().instance_url = url;
                Task::none()
            }
            Self::InstanceUrl(_) => {
                let instance = app.get_settings_view(&id).unwrap().instance_url.clone();
                Task::future(async {
                    if let Ok(x) = Data::get(Some(instance)).await {
                        *DATA.write().unwrap() = x;
                    }
                    Message::None
                })
                .chain(Application::update_data_cache())
            }
            Self::RemoveProviderInput(index) => {
                let _ = app
                    .get_settings_view(&id)
                    .unwrap()
                    .provider_inputs
                    .remove(index);
                Task::none()
            }
            Self::UpdateTheme(theme) => {
                app.cache.client_settings.theme =
                    Theme::ALL.iter().position(|x| x == &theme).unwrap_or(11);
                app.cache.client_settings.save();
                Task::none()
            }
            Self::UpdateUsePanes(x) => {
                app.cache.client_settings.use_panes = x;
                app.cache.client_settings.save();
                Task::none()
            }
        }
    }
}

async fn save_settings(settings: SettingsData) -> Message {
    let req = DATA.read().unwrap().to_request();
    let _: Value = req
        .make_request("settings/", &settings, RequestType::Put)
        .await
        .unwrap();
    Message::None
}

impl SettingsView {
    fn view_provider<'a>(id: u32, provider: Provider) -> Element<'a, Message> {
        let name = row![
            style::svg_button::danger("delete.svg", SUB_HEADING_SIZE).on_press(
                Message::HomePaneView(HomePaneViewMessage::Settings(
                    id,
                    SettingsViewMessage::DeleteProvider(provider.id.clone()),
                ))
            ),
            text(provider.name)
                .size(SUB_HEADING_SIZE)
                .style(style::text::primary)
        ]
        .align_y(Vertical::Center);

        let provider_type = text(provider.provider_type.to_string()).size(BODY_SIZE);

        let url = text(provider.url).size(BODY_SIZE);

        container(column![name, provider_type, url])
            .padding(Padding::new(20.0))
            .style(style::container::neutral_back)
            .into()
    }

    fn view_provider_input<'a>(
        id: u32,
        index: usize,
        input: &'a ProviderData,
    ) -> Element<'a, Message> {
        let name = row![
            text_input("Enter a name...", &input.name)
                .on_input(move |x| {
                    Message::HomePaneView(HomePaneViewMessage::Settings(
                        id,
                        SettingsViewMessage::UpdateProviderName(index, x),
                    ))
                })
                .size(SUB_HEADING_SIZE)
                .style(style::text_input::input),
            style::svg_button::primary("add.svg", SUB_HEADING_SIZE).on_press(
                Message::HomePaneView(HomePaneViewMessage::Settings(
                    id,
                    SettingsViewMessage::AddProvider(index),
                )),
            ),
            style::svg_button::danger("close.svg", SUB_HEADING_SIZE).on_press(
                Message::HomePaneView(HomePaneViewMessage::Settings(
                    id,
                    SettingsViewMessage::RemoveProviderInput(index),
                ))
            ),
        ];

        let url = text_input("Enter the provider url...", &input.url)
            .on_input(move |x| {
                Message::HomePaneView(HomePaneViewMessage::Settings(
                    id,
                    SettingsViewMessage::UpdateProviderUrl(index, x),
                ))
            })
            .size(SUB_HEADING_SIZE)
            .style(style::text_input::input);

        let api = text_input(
            "Enter your api key (if Ollama it is not used)...",
            &input.api_key,
        )
        .on_input(move |x| {
            Message::HomePaneView(HomePaneViewMessage::Settings(
                id,
                SettingsViewMessage::UpdateProviderKey(index, x),
            ))
        })
        .size(SUB_HEADING_SIZE)
        .secure(true)
        .style(style::text_input::input);

        let provider_type = pick_list(
            [
                ProviderType::OpenAI,
                ProviderType::Gemini,
                ProviderType::Ollama,
            ],
            Some(input.provider_type.clone()),
            move |x| {
                Message::HomePaneView(HomePaneViewMessage::Settings(
                    id,
                    SettingsViewMessage::UpdateProviderType(index, x),
                ))
            },
        )
        .style(style::pick_list::main)
        .menu_style(style::menu::main);

        container(column![name, url, api, provider_type])
            .padding(Padding::new(20.0))
            .style(style::container::neutral_back)
            .into()
    }

    pub fn themes<'a>(id: u32, current: Theme) -> Element<'a, Message> {
        let swatch = |color| {
            container(space::horizontal())
                .width(10)
                .height(10)
                .style(move |_theme: &Theme| {
                    container::Style::default()
                        .background(color)
                        .border(border::rounded(5))
                })
                .into()
        };

        let themes = grid(Theme::ALL.iter().map(|theme| {
            let palette = theme.palette();
            let selected = &current == theme;
            let cur = current.clone();

            let item = button(
                text(theme.to_string())
                    .width(Length::Fill)
                    .wrapping(text::Wrapping::None)
                    .center(),
            )
            .on_press(Message::HomePaneView(HomePaneViewMessage::Settings(
                id.clone(),
                SettingsViewMessage::UpdateTheme(theme.clone()),
            )))
            .padding([10, 0])
            .style(move |_theme, status| {
                let mut style = button::background(theme, status);

                style.border = style.border.rounded(5);

                if selected {
                    button::Style {
                        border: style.border.color(cur.palette().primary).width(3),
                        ..style
                    }
                } else {
                    style
                }
            });

            let swatches = {
                let colors = [palette.primary, palette.success, palette.danger];

                right_center(row(colors.into_iter().map(swatch)).spacing(5))
                    .padding(padding::right(10))
            };

            if &current == theme {
                float(stack![item, swatches]).scale(1.1).into()
            } else {
                hover(item, swatches)
            }
        }))
        .spacing(10)
        .fluid(300)
        .height(Length::Shrink);

        container(themes).into()
    }

    pub fn view<'a>(&'a self, app: &'a Application, id: u32) -> Element<'a, Message> {
        let sub_heading = |txt: &'static str| text(txt).size(BODY_SIZE).style(style::text::primary);
        let ochat = style::svg_input::primary(
            Some(String::from("link.svg")),
            text_input("Enter the instance url...", &self.instance_url)
                .on_input(move |x| {
                    Message::HomePaneView(HomePaneViewMessage::Settings(
                        id,
                        SettingsViewMessage::InstanceUrl(InputMessage::Update(x)),
                    ))
                })
                .on_submit(Message::HomePaneView(HomePaneViewMessage::Settings(
                    id,
                    SettingsViewMessage::InstanceUrl(InputMessage::Submit),
                ))),
            SUB_HEADING_SIZE,
        );

        let models_path = text(
            app.cache
                .settings
                .models_path
                .clone()
                .unwrap_or_default()
                .to_str()
                .unwrap()
                .to_string(),
        )
        .size(SUB_HEADING_SIZE)
        .style(style::text::text);

        let providers = {
            let header = row![
                text("Providers")
                    .size(BODY_SIZE)
                    .style(style::text::primary),
                style::svg_button::primary("add.svg", BODY_SIZE).on_press(Message::HomePaneView(
                    HomePaneViewMessage::Settings(id, SettingsViewMessage::AddProviderInput,)
                ))
            ]
            .width(Length::Shrink)
            .align_y(Vertical::Center);

            let body: Element<'a, Message> =
                if self.provider_inputs.is_empty() && DATA.read().unwrap().providers.is_empty() {
                    text("No providers found.").size(BODY_SIZE).into()
                } else {
                    let inputs = keyed_column(
                        self.provider_inputs
                            .iter()
                            .enumerate()
                            .map(|(i, provider)| (0, Self::view_provider_input(id, i, provider))),
                    )
                    .spacing(5);

                    let data = DATA.read().unwrap();
                    let providers = Scrollable::new(
                        row(data
                            .providers
                            .clone()
                            .into_iter()
                            .map(|provider| Self::view_provider(id, provider)))
                        .spacing(5),
                    )
                    .direction(Direction::Horizontal(Scrollbar::new()));

                    column![inputs, providers].spacing(5).into()
                };

            container(column![header, body])
        };

        let mut model_column = column([]).spacing(5);

        if let Ok(x) = DATA.read() {
            if !x.models.is_empty() {
                let preview_model = pick_list(
                    x.models.clone(),
                    app.cache.settings.previews_provider.clone(),
                    move |x| {
                        Message::HomePaneView(HomePaneViewMessage::Settings(
                            id,
                            SettingsViewMessage::UpdatePreviewModel(x),
                        ))
                    },
                )
                .style(style::pick_list::main)
                .menu_style(style::menu::main);

                model_column = model_column.push(sub_heading("Preview Model"));
                model_column = model_column.push(preview_model);

                let default_model = pick_list(
                    x.models.clone(),
                    app.cache.client_settings.default_provider.clone(),
                    move |x| {
                        Message::HomePaneView(HomePaneViewMessage::Settings(
                            id,
                            SettingsViewMessage::UpdateDefaultModel(x),
                        ))
                    },
                )
                .style(style::pick_list::main)
                .menu_style(style::menu::main);

                model_column = model_column.push(sub_heading("Default Model"));
                model_column = model_column.push(default_model);
            }
        }

        let use_panes = checkbox(app.cache.client_settings.use_panes)
            .label("Use Panes")
            .on_toggle(move |x| {
                Message::HomePaneView(HomePaneViewMessage::Settings(
                    id,
                    SettingsViewMessage::UpdateUsePanes(x),
                ))
            });

        container(
            column![
                sub_heading("Instance Url"),
                ochat,
                sub_heading("Models Download Path"),
                models_path,
                providers,
                model_column,
                sub_heading("Decorations"),
                use_panes,
                sub_heading("Themes"),
                container(Self::themes(id, app.theme()))
                    .padding(10)
                    .style(style::container::window_title_back)
            ]
            .spacing(10),
        )
        .into()
    }
}
