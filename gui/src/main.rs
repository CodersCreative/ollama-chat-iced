pub mod data;
pub mod pages;
pub mod style;
pub mod subscriptions;
pub mod utils;
pub mod windows;

use iced::{Element, Font, Subscription, Task, Theme, exit, widget::text, window};
use ochat_types::{
    chats::previews::Preview,
    settings::{Settings, SettingsData},
};
use std::{
    collections::BTreeMap,
    sync::{LazyLock, RwLock},
};

use crate::{
    pages::{
        Pages,
        home::{
            HomePage,
            panes::{
                data::{HomePaneSharedData, ModelsData, OptionsData, PromptsData},
                view::{
                    HomePaneViewData, HomePaneViewMessage, models::ModelsView,
                    options::OptionsView, prompts::PromptsView,
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
    pub const FONT: &[u8] = include_bytes!("../assets/RobotoMonoNerdFont-Regular.ttf");

    pub const HEADER_SIZE: u16 = 24;
    pub const SUB_HEADING_SIZE: u16 = 16;
    pub const BODY_SIZE: u16 = 12;
    pub const SMALL_SIZE: u16 = 8;
}

static DATA: LazyLock<RwLock<data::Data>> = LazyLock::new(|| {
    RwLock::new(
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(data::Data::get(None))
            .unwrap_or_default(),
    )
});

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
    let font = Font {
        family: iced::font::Family::Name("Roboto"),
        style: iced::font::Style::Normal,
        stretch: iced::font::Stretch::Normal,
        weight: iced::font::Weight::Normal,
    };
    iced::daemon(Application::title, Application::update, Application::view)
        .subscription(Application::subscription)
        .font(font::FONT)
        .default_font(font)
        .theme(|x, _| Application::theme(x))
        .run_with(Application::new)
}

#[derive(Debug, Clone)]
pub enum Message {
    None,
    Quit,
    Window(WindowMessage),
    HomePaneView(HomePaneViewMessage),
    Subscription(SubMessage),
    SetCache(AppCache),
}

impl Application {
    pub fn new() -> (Self, Task<Message>) {
        drop(DATA.read());
        let (_, open) = window::open(window::Settings::default());
        (
            Self {
                windows: BTreeMap::new(),
                cache: AppCache::default(),
                view_data: ViewData::default(),
                subscriptions: Subscriptions::default(),
            },
            open.map(|id| Message::Window(WindowMessage::WindowOpened(id)))
                .chain(Task::batch([Self::update_data_cache()])),
        )
    }

    pub fn update_data_cache() -> Task<Message> {
        Task::future(async {
            let req = DATA.read().unwrap().to_request();
            let mut cache = AppCache::default();

            cache.previews = req
                .make_request::<Vec<Preview>, ()>("preview/all/", &(), data::RequestType::Get)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|x| x.into())
                .collect();

            if let Ok(settings) = req
                .make_request::<Settings, ()>("settings/", &(), data::RequestType::Get)
                .await
            {
                cache.settings = settings.into();
            }

            cache.home_shared.models = ModelsData::get_ollama(None).await;
            cache.home_shared.prompts = PromptsData::get_prompts(None).await;
            cache.home_shared.options = OptionsData::get_gen_models(None).await;

            Message::SetCache(cache)
        })
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::None => Task::none(),
            Message::Window(message) => message.handle(self),
            Message::Subscription(message) => message.handle(self),
            Message::HomePaneView(message) => message.handle(self),
            Message::SetCache(cache) => {
                self.cache = cache;
                Task::none()
            }
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
        Theme::ALL[if let Some(theme) = &self.cache.settings.theme {
            theme.clone()
        } else {
            11
        }]
        .clone()
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

    pub fn get_prompts_view(&mut self, id: &u32) -> Option<&mut PromptsView> {
        self.view_data.home.prompts.get_mut(id)
    }

    pub fn get_options_view(&mut self, id: &u32) -> Option<&mut OptionsView> {
        self.view_data.home.options.get_mut(id)
    }
}
