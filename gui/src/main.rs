pub mod data;
pub mod pages;
pub mod style;
pub mod subscriptions;
pub mod utils;
pub mod windows;

use iced::{
    Element, Subscription, Task, Theme, clipboard, exit,
    widget::{markdown, text},
    window::{self},
};
use ochat_types::{
    chats::previews::Preview,
    settings::{Settings, SettingsData},
};
use std::{
    collections::BTreeMap,
    sync::{LazyLock, RwLock},
};

use crate::{
    data::{Data, settings::ClientSettings},
    font::get_iced_font,
    pages::{
        Pages,
        home::{
            HomePage,
            panes::{
                data::{HomePaneSharedData, ModelsData, OptionsData, PromptsData},
                view::{
                    HomePaneViewData, HomePaneViewMessage, models::ModelsView,
                    options::OptionsView, prompts::PromptsView, pulls::PullsView,
                    settings::SettingsView,
                },
            },
            sidebar::PreviewMk,
        },
        setup::SetupPage,
    },
    subscriptions::{SubMessage, Subscriptions},
    windows::{Window, message::WindowMessage},
};

pub mod font {
    use iced::Font;

    pub const FONT: &[u8] = include_bytes!("../assets/RobotoMonoNerdFont-Regular.ttf");
    pub fn get_iced_font() -> Font {
        Font {
            family: iced::font::Family::Name("Roboto"),
            style: iced::font::Style::Normal,
            stretch: iced::font::Stretch::Normal,
            weight: iced::font::Weight::Normal,
        }
    }
    pub const HEADER_SIZE: u32 = 24;
    pub const SUB_HEADING_SIZE: u32 = 16;
    pub const BODY_SIZE: u32 = 12;
    pub const SMALL_SIZE: u32 = 8;
}

static DATA: LazyLock<RwLock<data::Data>> = LazyLock::new(|| RwLock::new(data::Data::default()));

#[derive(Debug, Clone)]
pub struct Application {
    pub windows: BTreeMap<window::Id, Window>,
    pub cache: AppCache,
    pub view_data: ViewData,
    pub subscriptions: Subscriptions,
}

#[derive(Debug, Clone, Default)]
pub struct AppCache {
    pub previews: Vec<PreviewMk>,
    pub settings: SettingsData,
    pub client_settings: ClientSettings,
    pub home_shared: HomePaneSharedData,
}

#[derive(Debug, Clone, Default)]
pub struct ViewData {
    pub counter: u32,
    pub page_stack: Vec<Pages>,
    pub home: HomePaneViewData,
}

#[derive(Debug, Clone)]
pub enum InputMessage {
    Update(String),
    Submit,
}

fn main() -> iced::Result {
    iced::daemon(Application::new, Application::update, Application::view)
        .subscription(Application::subscription)
        .font(font::FONT)
        .title(Application::title)
        .default_font(get_iced_font())
        .theme(Application::window_theme)
        .run()
}

#[derive(Debug, Clone)]
pub enum Message {
    None,
    Quit,
    UriClicked(markdown::Uri),
    Window(WindowMessage),
    HomePaneView(HomePaneViewMessage),
    Subscription(SubMessage),
    Cache(CacheMessage),
    SaveToClipboard(String),
}

#[derive(Debug, Clone)]
pub enum CacheMessage {
    SetModels(ModelsData),
    SetPrompts(PromptsData),
    SetOptions(OptionsData),
    SetPreviews(Vec<PreviewMk>),
    SetSettings(SettingsData),
    SetTheme(Theme),
    SetInstanceUrl(String),
    SetUsePanes(bool),
}

impl CacheMessage {
    pub fn handle(self, app: &mut Application) -> Task<Message> {
        match self {
            Self::SetModels(x) => {
                app.cache.home_shared.models = x;
            }
            Self::SetPrompts(x) => {
                app.cache.home_shared.prompts = x;
            }
            Self::SetOptions(x) => {
                app.cache.home_shared.options = x;
            }
            Self::SetPreviews(x) => {
                app.cache.previews = x;
            }
            Self::SetSettings(x) => {
                app.cache.settings = x;
            }
            Self::SetTheme(theme) => {
                app.cache.client_settings.theme =
                    Theme::ALL.iter().position(|x| x == &theme).unwrap_or(11);
                app.cache.client_settings.save();
            }
            Self::SetUsePanes(x) => {
                app.cache.client_settings.use_panes = x;
                app.cache.client_settings.save();
            }
            Self::SetInstanceUrl(x) => {
                app.cache.client_settings.instance_url = x.clone();
                app.cache.client_settings.save();

                return Task::future(async {
                    if let Ok(x) = Data::get(Some(x)).await {
                        *DATA.write().unwrap() = x;
                    }
                    Message::None
                })
                .chain(Application::update_data_cache());
            }
        }

        Task::none()
    }
}

impl Application {
    pub fn new() -> (Self, Task<Message>) {
        let get_default_data = || -> Data {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(Data::get(None))
                .unwrap_or_default()
        };

        let mut cache = AppCache::default();

        if let Ok(x) = ClientSettings::load() {
            if &x.instance_url != "http://localhost:1212" && !x.instance_url.is_empty() {
                *DATA.write().unwrap() = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(Data::get(Some(x.instance_url.clone())))
                    .unwrap_or(get_default_data());
            } else {
                *DATA.write().unwrap() = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(Data::get(None))
                    .unwrap_or_default();
            }
            cache.client_settings = x;
        } else {
            *DATA.write().unwrap() = get_default_data();
        }

        let (_, open) = window::open(window::Settings::default());
        (
            Self {
                windows: BTreeMap::new(),
                cache,
                view_data: ViewData::default(),
                subscriptions: Subscriptions::default(),
            },
            open.map(|id| Message::Window(WindowMessage::WindowOpened(id)))
                .chain(Task::batch([Self::update_data_cache()])),
        )
    }

    pub fn update_data_cache() -> Task<Message> {
        Task::batch([
            Task::future(async {
                let req = DATA.read().unwrap().to_request();
                Message::Cache(CacheMessage::SetPreviews(
                    req.make_request::<Vec<Preview>, ()>(
                        "preview/all/",
                        &(),
                        data::RequestType::Get,
                    )
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .map(|x| x.into())
                    .collect(),
                ))
            }),
            Task::future(async {
                let req = DATA.read().unwrap().to_request();
                Message::Cache(CacheMessage::SetSettings(
                    if let Ok(settings) = req
                        .make_request::<Settings, ()>("settings/", &(), data::RequestType::Get)
                        .await
                    {
                        settings.into()
                    } else {
                        SettingsData::default()
                    },
                ))
            }),
            Task::future(async {
                Message::Cache(CacheMessage::SetPrompts(
                    PromptsData::get_prompts(None).await,
                ))
            }),
            Task::future(async {
                Message::Cache(CacheMessage::SetOptions(
                    OptionsData::get_gen_models(None).await,
                ))
            }),
            Task::future(async {
                Message::Cache(CacheMessage::SetModels(ModelsData::get_ollama(None).await))
            }),
        ])
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::None => Task::none(),
            Message::Window(message) => message.handle(self),
            Message::Subscription(message) => message.handle(self),
            Message::HomePaneView(message) => message.handle(self),
            Message::Cache(message) => message.handle(self),
            Message::UriClicked(x) => {
                open::that_in_background(x.to_string());
                Task::none()
            }
            Message::SaveToClipboard(x) => clipboard::write::<Message>(x.clone()),
            Message::Quit => exit(),
        }
    }

    pub fn title(&self, _window: window::Id) -> String {
        String::from("OChat")
    }

    pub fn view<'a>(&'a self, window_id: window::Id) -> Element<'a, Message> {
        if let Some(window) = self.windows.get(&window_id) {
            window.view(self, window_id)
        } else {
            text("Window Not Found").into()
        }
    }

    pub fn theme(&self) -> Theme {
        Theme::ALL[self.cache.client_settings.theme].clone()
    }

    pub fn window_theme(&self, _window_id: window::Id) -> Theme {
        Theme::ALL[self.cache.client_settings.theme].clone()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::close_events().map(|id| Message::Window(WindowMessage::WindowClosed(id))),
            self.subscriptions.get(self),
        ])
    }

    pub fn get_home_page(&mut self, id: &window::Id) -> Option<&mut HomePage> {
        let Pages::Home(ref mut page) = self.windows.get_mut(id).unwrap().page else {
            return None;
        };

        Some(page)
    }

    pub fn get_setup_page(&mut self, id: &window::Id) -> Option<&mut SetupPage> {
        let Pages::Setup(ref mut page) = self.windows.get_mut(id).unwrap().page else {
            return None;
        };

        Some(page)
    }

    pub fn get_models_view(&mut self, id: &u32) -> Option<&mut ModelsView> {
        self.view_data.home.models.get_mut(id)
    }

    pub fn get_settings_view(&mut self, id: &u32) -> Option<&mut SettingsView> {
        self.view_data.home.settings.get_mut(id)
    }

    pub fn get_pulls_view(&mut self, id: &u32) -> Option<&mut PullsView> {
        self.view_data.home.pulls.get_mut(id)
    }

    pub fn get_prompts_view(&mut self, id: &u32) -> Option<&mut PromptsView> {
        self.view_data.home.prompts.get_mut(id)
    }

    pub fn get_options_view(&mut self, id: &u32) -> Option<&mut OptionsView> {
        self.view_data.home.options.get_mut(id)
    }
}
