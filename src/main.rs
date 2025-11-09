#[cfg(feature = "voice")]
pub mod call;
pub mod chats;
pub mod common;
pub mod database;
pub mod download;
pub mod helper;
pub mod llm;
pub mod models;
pub mod options;
pub mod panes;
pub mod previews;
pub mod prompts;
pub mod providers;
pub mod save;
pub mod sidebar;
#[cfg(feature = "voice")]
pub mod sound;
pub mod start;
pub mod style;
pub mod tools;
pub mod update;
pub mod utils;
pub mod view;

use crate::{
    chats::chat::MarkdownMessage,
    previews::{PreviewResponse, SavedPreviews},
    providers::SavedProviders,
    save::Save,
    tools::SavedTools,
};
#[cfg(feature = "voice")]
use call::{Call, CallMessage};
use chats::{message::ChatsMessage, view::Chats, SavedChat, SavedChats, CHATS_FILE};
use common::Id;
use database::new_conn;
use download::{Download, DownloadProgress};
use iced::{
    clipboard, event,
    widget::{container, markdown, row, text_editor},
    window, Element, Event, Font, Subscription, Task, Theme,
};
use llm::{ChatProgress, ChatStreamId};
use models::{message::ModelsMessage, SavedModels};
#[cfg(feature = "sound")]
use natural_tts::{
    models::{gtts::GttsModel, tts_rs::TtsModel},
    NaturalTts, NaturalTtsBuilder,
};
use options::{message::OptionMessage, SavedOptions};
use panes::PaneMessage;
use panes::Panes;
use prompts::{message::PromptsMessage, SavedPrompts};
use save::SAVE_FILE;
use sidebar::SideBarState;
use update::Logic;
use view::View;

pub const FONT: &[u8] = include_bytes!("../assets/RobotoMonoNerdFont-Regular.ttf");
const PREVIEW_LEN: usize = 25;
const MIN_WIDTH: f32 = 300.0;

fn main() -> iced::Result {
    let _ = new_conn().unwrap();
    let font = Font {
        family: iced::font::Family::Name("Roboto"),
        style: iced::font::Style::Normal,
        stretch: iced::font::Stretch::Normal,
        weight: iced::font::Weight::Normal,
    };
    iced::application(ChatApp::title, ChatApp::update, ChatApp::view)
        .theme(ChatApp::theme)
        .font(FONT)
        .default_font(font)
        .subscription(ChatApp::subscription)
        .run()
}

pub struct ChatApp {
    pub save: Save,
    pub main_view: View,
    pub options: SavedOptions,
    pub model_info: SavedModels,
    pub providers: SavedProviders,
    pub prompts: SavedPrompts,
    pub previews: SavedPreviews,
    pub tools: SavedTools,
    pub chats: SavedChats,
    pub logic: Logic,
    pub panes: Panes,
    #[cfg(feature = "sound")]
    pub tts: NaturalTts,
    #[cfg(feature = "voice")]
    pub call: Call,
    pub potrait: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Pane(PaneMessage),
    #[cfg(feature = "voice")]
    Call(CallMessage),
    Models(ModelsMessage, Id),
    Prompts(PromptsMessage, Id),
    Option(OptionMessage, Id),
    Chats(ChatsMessage, Id),
    ChangeTheme(Theme),
    SaveToClipboard(String),
    RemoveChat(Id),
    URLClicked(markdown::Url),
    ChangeUsePanels(bool),
    ShowSettings,
    SideBar,
    Pulling((Id, Result<DownloadProgress, String>)),
    Generating((ChatStreamId, Result<ChatProgress, String>)),
    SetPreviews(Result<PreviewResponse, String>),
    StopGenerating(Id),
    Pull(String),
    StopPull(Id),
    EventOccurred(Event),
    None,
}

impl Default for ChatApp {
    fn default() -> Self {
        let (app, _) = Self::init();
        app
    }
}

impl ChatApp {
    fn _new() -> Self {
        Self::new_with_save(Save::default())
    }

    fn new_with_save(save: Save) -> Self {
        let providers = SavedProviders::default();
        Self {
            save,
            panes: Panes::new(panes::Pane::Chat(Id::new())),
            main_view: View::new(),
            logic: Logic::new(&providers),
            model_info: SavedModels::init().unwrap(),
            providers,
            options: SavedOptions::default(),
            prompts: SavedPrompts::default(),
            tools: SavedTools::default(),
            chats: SavedChats::default(),
            previews: SavedPreviews::default(),
            #[cfg(feature = "sound")]
            tts: NaturalTtsBuilder::default()
                .default_model(natural_tts::Model::Gtts)
                .gtts_model(GttsModel::default())
                .tts_model(TtsModel::default())
                .build()
                .unwrap(),
            #[cfg(feature = "voice")]
            call: Call::new("".to_string()),
            potrait: false,
        }
    }

    fn init() -> (ChatApp, Task<Message>) {
        let mut app = Self::new_with_save(match Save::load(SAVE_FILE) {
            Ok(x) => x,
            Err(_) => {
                let save = Save::default();
                save.save(SAVE_FILE);
                save
            }
        });

        if let Ok(x) = SavedOptions::load(options::SETTINGS_FILE) {
            app.options = x;
        } else {
            app.options.save(options::SETTINGS_FILE);
        }

        if let Ok(x) = SavedPrompts::load(prompts::PROMPTS_PATH) {
            app.prompts = x;
        } else {
            app.prompts.save(prompts::PROMPTS_PATH);
        }

        if let Ok(x) = SavedProviders::load(providers::PROVIDERS_FILE) {
            app.logic = Logic::new(&x);
            app.providers = x;
        } else {
            app.providers.save(providers::PROVIDERS_FILE);
        }

        if let Ok(x) = SavedTools::load(tools::TOOLS_PATH) {
            app.tools = x;
        } else {
            app.tools.save(tools::TOOLS_PATH);
        }

        if let Ok(x) = SavedChats::load(chats::CHATS_FILE) {
            app.chats = x;
        } else {
            app.chats.save(chats::CHATS_FILE);
        }

        if let Ok(x) = SavedPreviews::load(previews::PREVIEWS_FILE) {
            app.previews = x;
        } else {
            app.previews.save(previews::PREVIEWS_FILE);
        }

        if let Some(i) = app.save.theme {
            app.main_view.set_theme(Theme::ALL[i].clone());
        }

        if !app.logic.models.is_empty() {
            if app.chats.0.is_empty() {
                app.chats.0.insert(Id::new(), SavedChat::default());
            }

            if let Some(saved) = app.chats.0.iter().last() {
                let first = (
                    Id::new(),
                    Chats::new(
                        if let Some(model) = app.logic.models.first() {
                            vec![model.clone()]
                        } else {
                            Vec::new()
                        },
                        saved.0.clone(),
                        saved.1.to_mk(&saved.1.default_chats),
                        saved.1.default_chats.clone(),
                        saved.1.default_tools.clone(),
                    ),
                );

                app.panes = Panes::new(panes::Pane::Chat(first.0.clone()));
                app.panes.last_chat = first.0.clone();
                app.main_view.add_to_chats(first.0, first.1);
            }

            if let Some(model) = app.logic.models.first() {
                app.options.get_create_model_options_index(model.clone());
            }
        } else {
            let model = panes::Pane::new_models(&mut app);
            app.panes = Panes::new(model);
        }

        if app.previews.0.len() != app.chats.0.len() {
            let task = app.regenerate_side_chats(Vec::new());
            (app, task)
        } else {
            (app, Task::none())
        }
    }

    fn title(&self) -> String {
        String::from("OChat")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Option(x, k) => x.handle(k, self),
            Message::ChangeUsePanels(x) => {
                self.save.use_panes = x;
                self.save.save(SAVE_FILE);
                Task::none()
            }
            Message::Models(x, k) => x.handle(k, self),
            Message::Prompts(x, k) => x.handle(k, self),
            Message::None => Task::none(),
            Message::SaveToClipboard(x) => clipboard::write::<Message>(x.clone()),
            Message::Chats(x, i) => x.handle(i, self),
            Message::Pane(x) => x.handle(self),
            #[cfg(feature = "voice")]
            Message::Call(x) => x.handle(self),
            Message::URLClicked(x) => {
                open::that_in_background(x.to_string());
                Task::none()
            }
            Message::EventOccurred(event) => {
                if let Event::Window(window::Event::CloseRequested) = event {
                    window::get_latest().and_then(window::close)
                } else if let Event::Window(window::Event::Resized(x)) = event {
                    self.potrait = x.width < MIN_WIDTH;
                    Task::none()
                } else {
                    Task::none()
                }
            }
            Message::StopPull(id) => {
                self.main_view.remove_download_by_id(&id);
                Task::none()
            }
            Message::StopGenerating(id) => {
                self.main_view.update_chat_streams(|streams| {
                    let ids: Vec<ChatStreamId> = streams
                        .iter()
                        .filter(|x| x.0 .0 == id)
                        .map(|x| x.0.clone())
                        .collect();

                    for id in ids {
                        streams.remove(&id);
                    }
                });

                self.main_view.update_chat_by_saved(&id, |x| {
                    x.set_state(chats::view::State::Idle);
                });
                Task::none()
            }
            Message::ShowSettings => {
                self.toggle_side_bar_state(SideBarState::Settings);
                Task::none()
            }
            Message::SideBar => {
                self.toggle_side_bar_state(SideBarState::Hidden);
                Task::none()
            }
            Message::RemoveChat(x) => return self.remove_chat(x),
            Message::ChangeTheme(x) => self.change_theme(x.clone()),
            Message::Pull(x) => {
                self.main_view.add_download(
                    Id::new(),
                    Download::new(x.clone(), self.logic.get_random_provider().unwrap()),
                );
                Task::none()
            }
            Message::SetPreviews(preview) => {
                if let Ok(preview) = preview {
                    if let Some(p) = self.previews.0.get_mut(&preview.chat) {
                        p.text = preview.text;
                    }

                    self.main_view
                        .set_side_chats(self.previews.get_side_chats());
                    self.previews.save(previews::PREVIEWS_FILE);
                }

                Task::none()
            }
            Message::Generating((id, progress)) => {
                self.main_view.update_chat_streams(|streams| {
                    if let Some(chat) = streams.get_mut(&id) {
                        chat.progress(progress.clone());
                    }
                });

                let mut mk: Vec<MarkdownMessage> = Vec::new();

                if let Ok(ChatProgress::Generating(progress, _tools)) = progress {
                    let path = self
                        .main_view
                        .chats()
                        .iter()
                        .find(|x| x.1.saved_chat() == &id.0 && x.1.chats().contains(&id.1))
                        .map(|x| x.1.chats());

                    if let Some(chat) = self.chats.0.get_mut(&id.0) {
                        if let Some(message) = chat.chats.chats.get_mut(id.1) {
                            message.add_to_content(progress.content.as_str());
                        }

                        if let Some(path) = path {
                            mk = chat.to_mk(&path);
                        }
                    }

                    self.main_view
                        .update_chat_by_saved_and_message(&id.0, &id.1, |chat| {
                            chat.set_markdown(mk.clone());
                            chat.set_state(chats::view::State::Generating);
                        });
                } else if let Ok(ChatProgress::Finished) = progress {
                    let _ = self.main_view.chat_streams_mut().remove(&id);

                    if let Some(chat) = self.chats.0.get_mut(&id.0) {
                        if let Some(message) = chat.chats.chats.get_mut(id.1) {
                            if message.content().contains("<think>") {
                                let c = message.content().clone();
                                let split = c.split_once("<think>").unwrap();
                                let mut content = split.0.to_string();
                                let think = if split.1.contains("</think>") {
                                    let split2 = split.1.rsplit_once("</think>").unwrap();
                                    content.push_str(split2.1);
                                    split2.0.to_string()
                                } else {
                                    split.1.to_string()
                                };

                                message.set_content(content.trim().to_string());
                                if !think.trim().is_empty() {
                                    message.set_thinking(Some(think.trim().to_string()));
                                }
                            }

                            let path = self
                                .main_view
                                .chats()
                                .iter()
                                .find(|x| x.1.saved_chat() == &id.0 && x.1.chats().contains(&id.1))
                                .map(|x| x.1.chats());

                            if let Some(path) = path {
                                mk = chat.to_mk(&path);

                                self.chats.save(CHATS_FILE);
                                self.main_view.update_chat_by_saved_and_message(
                                    &id.0,
                                    &id.1,
                                    |chat| {
                                        chat.set_content(text_editor::Content::new());
                                        chat.set_images(Vec::new());
                                        chat.set_state(chats::view::State::Idle);
                                        chat.set_markdown(mk.clone());
                                    },
                                );
                                return self.regenerate_side_chats(vec![id.0]);
                            }
                        }
                    }

                    self.chats.save(CHATS_FILE);
                    self.main_view
                        .update_chat_by_saved_and_message(&id.0, &id.1, |chat| {
                            chat.set_content(text_editor::Content::new());
                            chat.set_images(Vec::new());
                            chat.set_state(chats::view::State::Idle);
                        });
                    return self.regenerate_side_chats(vec![id.0]);
                }

                Task::none()
            }
            Message::Pulling((id, progress)) => {
                if let Ok(progress) = progress.clone() {
                    if let DownloadProgress::Finished = progress {
                        self.main_view.remove_download_by_id(&id);
                        self.logic.update_all_models();
                        return Task::none();
                    }
                }

                self.main_view.update_downloads(move |x| {
                    if let Some(download) = x.get_mut(&id) {
                        download.progress(progress.clone());
                    }
                });
                Task::none()
            }
        }
    }

    fn view<'a>(&'a self) -> Element<'a, Message> {
        if self.potrait && self.main_view.side_state() != &SideBarState::Hidden {
            return self.main_view.side_bar(self);
        }
        container(row![self.main_view.side_bar(self), self.panes.view(self),]).into()
    }

    fn theme(&self) -> iced::Theme {
        self.main_view.theme().clone()
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut actions: Vec<Subscription<Message>> = self
            .main_view
            .downloads()
            .iter()
            .map(|(i, x)| x.subscription(self, i.clone()))
            .collect();
        self.main_view
            .chat_streams()
            .iter()
            .for_each(|(i, x)| actions.push(x.subscription(self, i.clone())));
        actions.push(event::listen().map(Message::EventOccurred));
        Subscription::batch(actions)
    }
}
