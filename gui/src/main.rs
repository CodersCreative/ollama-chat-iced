pub mod data;
pub mod pages;
pub mod style;
pub mod subscriptions;
pub mod utils;
pub mod windows;

use iced::{
    Element, Length, Subscription, Task, Theme,
    alignment::Vertical,
    clipboard, exit,
    widget::{
        column, container, markdown, mouse_area, progress_bar, right, row, rule, stack, text,
    },
    window::{self},
};
use ochat_types::{
    chats::previews::Preview,
    providers::{hf::HFPullModelStreamResult, ollama::OllamaPullModelStreamResult},
    settings::{Settings, SettingsData},
};
use std::{
    collections::BTreeMap,
    fmt::Debug,
    rc::Rc,
    sync::{LazyLock, RwLock},
};

use crate::{
    data::{Data, settings::ClientSettings},
    font::{BODY_SIZE, SUB_HEADING_SIZE, get_iced_font},
    pages::{
        Pages,
        home::{
            HomePage,
            panes::{
                data::{HomePaneSharedData, MessageMk, ModelsData, OptionsData, PromptsData},
                view::{
                    HomePaneViewData, HomePaneViewMessage, chat::ChatsView, models::ModelsView,
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
    pub popups: Vec<PopUp>,
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

#[derive(Clone)]
pub enum PopUp {
    Err(String),
    Custom(Rc<dyn Fn(&Application) -> Element<Message>>),
}

impl Debug for PopUp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pop Up")
    }
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
    Batch(Vec<Self>),
    RemovePopUp(usize),
    Err(String),
}

#[derive(Debug, Clone)]
pub enum CacheMessage {
    AddPreview(PreviewMk),
    AddMessage(MessageMk),
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
            Self::AddPreview(x) => {
                app.cache.previews.retain(|y| y.id != x.id);
                app.cache.previews.push(x);
            }
            Self::AddMessage(x) => {
                app.cache
                    .home_shared
                    .messages
                    .0
                    .insert(x.base.id.key().to_string(), x);
            }
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
                *DATA.write().unwrap() = get_default_data();
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
                popups: Vec::new(),
            },
            open.map(|id| Message::Window(WindowMessage::WindowOpened(id)))
                .chain(Task::batch([Self::update_data_cache()])),
        )
    }

    pub fn update_data_cache() -> Task<Message> {
        Task::batch([
            Task::future(async {
                let req = DATA.read().unwrap().to_request();
                match req
                    .make_request::<Vec<Preview>, ()>("preview/all/", &(), data::RequestType::Get)
                    .await
                {
                    Ok(x) => Message::Cache(CacheMessage::SetPreviews(
                        x.into_iter().map(|x| x.into()).collect(),
                    )),
                    Err(e) => Message::Err(e),
                }
            }),
            Task::future(async {
                let req = DATA.read().unwrap().to_request();
                match req
                    .make_request::<Settings, ()>("settings/", &(), data::RequestType::Get)
                    .await
                {
                    Ok(x) => Message::Cache(CacheMessage::SetSettings(x.into())),
                    Err(e) => Message::Err(e),
                }
            }),
            Task::future(async {
                match PromptsData::get(None).await {
                    Ok(x) => Message::Cache(CacheMessage::SetPrompts(x)),
                    Err(e) => Message::Err(e),
                }
            }),
            Task::future(async {
                match OptionsData::get(None).await {
                    Ok(x) => Message::Cache(CacheMessage::SetOptions(x)),
                    Err(e) => Message::Err(e),
                }
            }),
            Task::future(async {
                match ModelsData::get(None).await {
                    Ok(x) => Message::Cache(CacheMessage::SetModels(x)),
                    Err(e) => Message::Err(e),
                }
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
            Message::Err(e) => {
                self.add_popup(PopUp::Err(e));
                Task::none()
            }
            Message::UriClicked(x) => {
                open::that_in_background(x.to_string());
                Task::none()
            }
            Message::RemovePopUp(index) => {
                self.popups.remove(index);
                Task::none()
            }
            Message::Batch(messages) => Task::batch(messages.into_iter().map(|x| Task::done(x))),
            Message::SaveToClipboard(x) => clipboard::write::<Message>(x.clone()),
            Message::Quit => exit(),
        }
    }

    pub fn title(&self, _window: window::Id) -> String {
        String::from("OChat")
    }

    pub fn view<'a>(&'a self, window_id: window::Id) -> Element<'a, Message> {
        let body = if let Some(window) = self.windows.get(&window_id) {
            window.view(self, window_id)
        } else {
            text("Window Not Found").into()
        };

        if !self.popups.is_empty() {
            let popups = right(
                column({
                    let mut widgets: Vec<Element<Message>> = self
                        .popups
                        .iter()
                        .enumerate()
                        .map(|(i, x)| {
                            mouse_area(
                                container(match x {
                                    PopUp::Err(e) => {
                                        text(e).style(style::text::danger).size(BODY_SIZE).into()
                                    }
                                    PopUp::Custom(x) => x(self),
                                })
                                .width(Length::Fill)
                                .padding(10)
                                .style(style::container::chat_back),
                            )
                            .on_press(Message::RemovePopUp(i))
                            .into()
                        })
                        .collect();

                    widgets.push(rule::horizontal(2).style(style::rule::primary).into());

                    widgets
                })
                .spacing(10)
                .height(Length::Shrink)
                .width(400),
            );

            stack![body, popups].into()
        } else {
            body
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

    pub fn unwrap_value<T: Default>(&mut self, value: Result<T, String>) -> T {
        match value {
            Ok(x) => x,
            Err(e) => {
                self.add_popup(PopUp::Err(e));
                T::default()
            }
        }
    }

    pub fn add_popup(&mut self, popup: PopUp) {
        self.popups.push(popup);
    }

    pub fn add_pull_pop_up(&mut self, id: u32) {
        self.popups.push(PopUp::Custom(Rc::new(move |app| {
            let danger_text = |txt: String| text(txt).style(style::text::danger).size(BODY_SIZE);
            let text_text = |txt: String| text(txt).style(style::text::text).size(BODY_SIZE);
            let primary_text = |txt: String| text(txt).style(style::text::primary).size(BODY_SIZE);
            let progress = |progress: f32| {
                row![
                    progress_bar(0.0..=100.0, progress)
                        .length(Length::Fill)
                        .girth(Length::Fixed(SUB_HEADING_SIZE as f32)),
                    text(format!("{:.2}%", progress))
                        .style(style::text::primary)
                        .width(75.0)
                        .size(SUB_HEADING_SIZE)
                ]
                .align_y(Vertical::Center)
                .spacing(20)
            };

            if let Some(pull) = app.subscriptions.ollama_pulls.get(&id) {
                let body: Element<Message> = match &pull.state {
                    OllamaPullModelStreamResult::Err(e) => danger_text(e.to_string()).into(),
                    OllamaPullModelStreamResult::Finished => {
                        primary_text("Pull Finished!".to_string()).into()
                    }
                    OllamaPullModelStreamResult::Idle => {
                        text_text("Starting...".to_string()).into()
                    }
                    _ => progress(pull.get_percent()).into(),
                };

                column![primary_text(pull.model.clone()), body].into()
            } else if let Some(pull) = app.subscriptions.hf_pulls.get(&id) {
                let body: Element<Message> = match &pull.state {
                    HFPullModelStreamResult::Err(e) => danger_text(e.to_string()).into(),
                    HFPullModelStreamResult::Finished => {
                        primary_text("Pull Finished!".to_string()).into()
                    }
                    HFPullModelStreamResult::Idle => text_text("Starting...".to_string()).into(),
                    _ => progress(pull.get_percent()).into(),
                };

                column![primary_text(pull.model.clone()), body].into()
            } else {
                text("Pull Finished!")
                    .style(style::text::primary)
                    .size(BODY_SIZE)
                    .into()
            }
        })));
    }

    pub fn get_models_view(&mut self, id: &u32) -> Option<&mut ModelsView> {
        self.view_data.home.models.get_mut(id)
    }

    pub fn get_chats_view(&mut self, id: &u32) -> Option<&mut ChatsView> {
        self.view_data.home.chats.get_mut(id)
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
