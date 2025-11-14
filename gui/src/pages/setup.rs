use crate::{
    Application, DATA, Message,
    data::RequestType,
    font::{BODY_SIZE, HEADER_SIZE, SUB_HEADING_SIZE},
    pages::{PageMessage, Pages},
    style,
    windows::message::WindowMessage,
};
use iced::{
    Element, Length, Padding, Task,
    alignment::Vertical,
    widget::{center, column, container, keyed_column, pick_list, row, text, text_input},
    window,
};
use ochat_types::{
    providers::{Provider, ProviderData, ProviderDataBuilder, ProviderType},
    settings::{SettingsProvider, SettingsProviderBuilder},
    surreal::RecordId,
};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct SetupPage {
    pub provider_inputs: Vec<ProviderData>,
    pub previews_model: Option<SettingsProvider>,
    pub default_model: Option<SettingsProvider>,
    pub tools_model: Option<SettingsProvider>,
    pub use_panes: bool,
    pub theme: usize,
}

#[derive(Debug, Clone)]
pub enum SetupMessage {
    UpdateProviderName(usize, String),
    UpdateProviderUrl(usize, String),
    UpdateProviderType(usize, ProviderType),
    UpdateProviderKey(usize, String),
    UpdatePreviewModel(SettingsProvider),
    UpdateDefaultModel(SettingsProvider),
    UpdateToolsModel(SettingsProvider),
    UpdateInstanceUrl(String),
    UpdateUsePanes(bool),
    UpdateTheme(usize),
    DeleteProvider(RecordId),
    RemoveProviderInput(usize),
    AddProvider(usize),
    AddProviderInput,
}

impl SetupMessage {
    pub fn handle(self, app: &mut Application, id: window::Id) -> Task<Message> {
        let Pages::Setup(ref mut page) = app.windows.get_mut(&id).unwrap().page else {
            return Task::none();
        };

        macro_rules! UpdateProviderInputProperty {
            ($index:expr, $prop:ident) => {{
                page.provider_inputs.get_mut($index).unwrap().$prop = $prop;
                Task::none()
            }};
        }

        macro_rules! UpdateModel {
            ($model:expr, $prop:ident) => {{
                page.$prop = Some($model);
                Task::none()
            }};
        }

        match self {
            Self::AddProviderInput => {
                page.provider_inputs.push(
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
                let input = page.provider_inputs.remove(index);
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
            Self::UpdatePreviewModel(model) => UpdateModel!(model, previews_model),
            Self::UpdateDefaultModel(model) => UpdateModel!(model, default_model),
            Self::UpdateToolsModel(model) => UpdateModel!(model, tools_model),
            Self::UpdateInstanceUrl(url) => {
                // TODO Find an easy way to cache this value as to not send too many requests
                Task::none()
            }
            Self::RemoveProviderInput(index) => {
                let _ = page.provider_inputs.remove(index);
                Task::none()
            }
            Self::UpdateTheme(index) => {
                // TODO Add Theme Switching
                Task::none()
            }
            Self::UpdateUsePanes(x) => {
                // TODO Add Panes
                Task::none()
            }
        }
    }
}

fn view_provider<'a>(id: window::Id, provider: Provider) -> Element<'a, Message> {
    let name = row![
        text(provider.name).size(SUB_HEADING_SIZE),
        style::button::svg_button("delete.svg", SUB_HEADING_SIZE).on_press(Message::Window(
            WindowMessage::Page(
                id.clone(),
                PageMessage::Setup(SetupMessage::DeleteProvider(provider.id.clone())),
            )
        ))
    ];

    let url = text(provider.url).size(BODY_SIZE);

    let provider_type = text(provider.provider_type.to_string());

    container(column![name, url, provider_type])
        .padding(Padding::new(20.0))
        .style(style::container::neutral_back)
        .into()
}

fn view_provider_input<'a>(
    id: window::Id,
    index: usize,
    input: &'a ProviderData,
) -> Element<'a, Message> {
    let name = row![
        text_input("Enter a name...", &input.name)
            .on_input(move |x| {
                Message::Window(WindowMessage::Page(
                    id.clone(),
                    PageMessage::Setup(SetupMessage::UpdateProviderName(index, x)),
                ))
            })
            .size(SUB_HEADING_SIZE)
            .style(style::text_input::input),
        style::button::svg_button("close.svg", SUB_HEADING_SIZE).on_press(Message::Window(
            WindowMessage::Page(
                id.clone(),
                PageMessage::Setup(SetupMessage::RemoveProviderInput(index)),
            )
        )),
        style::button::svg_button("add.svg", SUB_HEADING_SIZE).on_press(Message::Window(
            WindowMessage::Page(
                id.clone(),
                PageMessage::Setup(SetupMessage::AddProvider(index)),
            )
        ))
    ];

    let url = text_input("Enter the provider url...", &input.url)
        .on_input(move |x| {
            Message::Window(WindowMessage::Page(
                id.clone(),
                PageMessage::Setup(SetupMessage::UpdateProviderUrl(index, x)),
            ))
        })
        .size(SUB_HEADING_SIZE)
        .style(style::text_input::input);

    let api = text_input(
        "Enter your api key (if Ollama it is not used)...",
        &input.api_key,
    )
    .on_input(move |x| {
        Message::Window(WindowMessage::Page(
            id.clone(),
            PageMessage::Setup(SetupMessage::UpdateProviderKey(index, x)),
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
            Message::Window(WindowMessage::Page(
                id,
                PageMessage::Setup(SetupMessage::UpdateProviderType(index, x)),
            ))
        },
    );

    container(column![name, url, api, provider_type])
        .padding(Padding::new(20.0))
        .style(style::container::neutral_back)
        .into()
}

impl SetupPage {
    pub fn view<'a>(&'a self, app: &'a Application, id: window::Id) -> Element<'a, Message> {
        let banner = text("Welcome to OChat!")
            .size(HEADER_SIZE)
            .style(style::text::primary);

        let ochat = text_input(
            "Enter the instance url...",
            &DATA
                .read()
                .unwrap()
                .instance_url
                .clone()
                .unwrap_or_default(),
        )
        .on_input(move |x| {
            Message::Window(WindowMessage::Page(
                id,
                PageMessage::Setup(SetupMessage::UpdateInstanceUrl(x)),
            ))
        })
        .size(SUB_HEADING_SIZE)
        .style(style::text_input::input);

        let providers = {
            let header = row![
                text("Providers")
                    .size(BODY_SIZE)
                    .style(style::text::primary),
                style::button::svg_button("add.svg", BODY_SIZE).on_press(Message::Window(
                    WindowMessage::Page(
                        id.clone(),
                        PageMessage::Setup(SetupMessage::AddProviderInput),
                    )
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
                            .map(|(i, provider)| (0, view_provider_input(id, i, provider))),
                    )
                    .spacing(5);

                    let data = DATA.read().unwrap();
                    let providers = keyed_column(
                        data.providers
                            .clone()
                            .into_iter()
                            .map(|provider| (0, view_provider(id, provider))),
                    )
                    .spacing(5);

                    column![inputs, providers].into()
                };

            container(column![header, body])
        };

        let mut model_column = column([]).spacing(5);

        if let Ok(x) = DATA.read() {
            if !x.models.is_empty() {
                let preview_model =
                    pick_list(x.models.clone(), self.previews_model.clone(), move |x| {
                        Message::Window(WindowMessage::Page(
                            id,
                            PageMessage::Setup(SetupMessage::UpdatePreviewModel(x)),
                        ))
                    });
                model_column = model_column.push(preview_model);

                let default_model =
                    pick_list(x.models.clone(), self.default_model.clone(), move |x| {
                        Message::Window(WindowMessage::Page(
                            id,
                            PageMessage::Setup(SetupMessage::UpdateDefaultModel(x)),
                        ))
                    });
                model_column = model_column.push(default_model);

                let tools_model = pick_list(x.models.clone(), self.tools_model.clone(), move |x| {
                    Message::Window(WindowMessage::Page(
                        id,
                        PageMessage::Setup(SetupMessage::UpdateToolsModel(x)),
                    ))
                });
                model_column = model_column.push(tools_model);
            }
        }

        center(
            container(column![banner, ochat, providers, model_column,].spacing(10))
                .width(Length::Shrink)
                .padding(Padding::new(20.0))
                .style(style::container::neutral_back),
        )
        .into()
    }
}
