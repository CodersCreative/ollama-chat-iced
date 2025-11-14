use crate::{
    Application, Message,
    font::{BODY_SIZE, HEADER_SIZE, SUB_HEADING_SIZE},
    style,
};
use iced::{
    Element, Length, Padding,
    alignment::Vertical,
    widget::{center, column, container, keyed_column, pick_list, row, text, text_input},
};
use ochat_types::{
    providers::{Provider, ProviderData, ProviderType},
    settings::SettingsProvider,
};

#[derive(Debug, Clone, Default)]
pub struct SetupPage {
    pub provider_inputs: Vec<ProviderData>,
    pub previews_model: Option<SettingsProvider>,
    pub default_model: Option<SettingsProvider>,
    pub tools_model: Option<SettingsProvider>,
    pub use_panes: bool,
    pub theme: usize,
}

pub enum SetupMessage {
    UpdateProviderName(usize, String),
    UpdateProviderUrl(usize, String),
    UpdateProviderType(usize, ProviderType),
    UpdateProviderKey(usize, String),
    UpdatePreviewModel(SettingsProvider),
    UpdateDefaultModel(SettingsProvider),
    UpdateToolsModel(SettingsProvider),
    UpdateUsePanes(bool),
    UpdateTheme(usize),
    DeleteProvider(),
    AddProvider,
}

fn view_provider<'a>(provider: &'a Provider) -> Element<'a, Message> {
    let name = row![
        text(&provider.name).size(SUB_HEADING_SIZE),
        style::button::svg_button("delete.svg", SUB_HEADING_SIZE)
    ];

    let url = text(&provider.url).size(BODY_SIZE);

    let provider_type = text(provider.provider_type.to_string());

    container(column![name, url, provider_type])
        .padding(Padding::new(20.0))
        .style(style::container::neutral_back)
        .into()
}

fn view_provider_input<'a>(index: usize, input: &'a ProviderData) -> Element<'a, Message> {
    let name = row![
        text_input("Enter a name...", &input.name)
            .size(SUB_HEADING_SIZE)
            .style(style::text_input::input),
        style::button::svg_button("close.svg", SUB_HEADING_SIZE)
    ];

    let url = text_input("Enter the provider url...", &input.name)
        .size(SUB_HEADING_SIZE)
        .style(style::text_input::input);

    let api = text_input(
        "Enter your api key (if Ollama it is not used)...",
        &input.name,
    )
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
        |_| Message::None,
    );

    container(column![name, url, api, provider_type])
        .padding(Padding::new(20.0))
        .style(style::container::neutral_back)
        .into()
}

impl SetupPage {
    pub fn view<'a>(&'a self, app: &'a Application) -> Element<'a, Message> {
        let banner = text("Welcome to OChat!")
            .size(HEADER_SIZE)
            .style(style::text::primary);

        let ochat = text_input(
            "Enter the instance url...",
            &app.data.instance_url.clone().unwrap_or_default(),
        )
        .size(SUB_HEADING_SIZE)
        .style(style::text_input::input);

        let providers = {
            let header = row![
                text("Providers")
                    .size(BODY_SIZE)
                    .style(style::text::primary),
                style::button::svg_button("add.svg", BODY_SIZE)
            ]
            .width(Length::Shrink)
            .align_y(Vertical::Center);

            let body: Element<'a, Message> =
                if self.provider_inputs.is_empty() && app.data.providers.is_empty() {
                    text("No providers found.").size(BODY_SIZE).into()
                } else {
                    let inputs = keyed_column(
                        self.provider_inputs
                            .iter()
                            .enumerate()
                            .map(|(i, provider)| (0, view_provider_input(i, provider))),
                    )
                    .spacing(5);

                    let providers = keyed_column(
                        app.data
                            .providers
                            .iter()
                            .map(|provider| (0, view_provider(provider))),
                    )
                    .spacing(5);

                    column![inputs, providers].into()
                };

            container(column![header, body])
        };

        let preview_model = pick_list(app.data.models.clone(), self.previews_model.clone(), |_| {
            Message::None
        });

        let default_model = pick_list(app.data.models.clone(), self.default_model.clone(), |_| {
            Message::None
        });

        let tools_model = pick_list(app.data.models.clone(), self.tools_model.clone(), |_| {
            Message::None
        });

        center(
            container(
                column![
                    banner,
                    ochat,
                    providers,
                    preview_model,
                    default_model,
                    tools_model
                ]
                .spacing(10),
            )
            .width(Length::Shrink)
            .padding(Padding::new(20.0))
            .style(style::container::neutral_back),
        )
        .into()
    }
}
