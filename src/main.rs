#[cfg(feature = "voice")]
pub mod call;
pub mod chats;
pub mod common;
pub mod download;
pub mod helper;
pub mod llm;
pub mod models;
pub mod options;
pub mod panes;
pub mod prompts;
pub mod save;
pub mod sidebar;
#[cfg(feature = "voice")]
pub mod sound;
pub mod start;
pub mod style;
pub mod update;
pub mod utils;
pub mod view;

use crate::save::Save;
#[cfg(feature = "voice")]
use call::{Call, CallMessage};
use chats::{
    chat::{Chat, ChatBuilder},
    message::ChatsMessage,
    tree::Reason,
    view::Chats,
    SavedChat, SavedChats, CHATS_FILE,
};
use common::Id;
use download::{Download, DownloadProgress};
use iced::{
    clipboard, event,
    widget::{combo_box, container, markdown, row, text_editor},
    window, Element, Event, Font, Subscription, Task, Theme,
};
use llm::ChatProgress;
use models::{message::ModelsMessage, SavedModels};
use natural_tts::{
    models::{gtts::GttsModel, tts_rs::TtsModel},
    NaturalTts, NaturalTtsBuilder,
};
use ollama_rs::generation::chat::ChatMessage;
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
    pub prompts: SavedPrompts,
    pub chats: SavedChats,
    pub logic: Logic,
    pub panes: Panes,
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
    Generating((Id, Result<ChatProgress, String>)),
    Generated(Id, Result<ChatMessage, String>, Option<String>),
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
    fn new() -> Self {
        Self::new_with_save(Save::default())
    }

    fn new_with_save(save: Save) -> Self {
        Self {
            save,
            panes: Panes::new(panes::Pane::Chat(Id::new())),
            main_view: View::new(),
            logic: Logic::new(),
            model_info: SavedModels::init().unwrap(),
            options: SavedOptions::default(),
            prompts: SavedPrompts::default(),
            chats: SavedChats::default(),
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
            Err(_) => Save::default(),
        });

        app.options = match SavedOptions::load(options::SETTINGS_FILE) {
            Ok(x) => x,
            Err(_) => SavedOptions::default(),
        };

        app.prompts = match SavedPrompts::load(prompts::PROMPTS_PATH) {
            Ok(x) => x,
            Err(_) => SavedPrompts::default(),
        };

        app.chats = match SavedChats::load(chats::CHATS_FILE) {
            Ok(x) => x,
            Err(_) => SavedChats::default(),
        };

        let models = app.logic.get_models();
        app.logic.combo_models = combo_box::State::new(models.clone());

        if let Some(i) = app.save.theme {
            app.main_view.set_theme(Theme::ALL[i].clone());
        }

        if !models.is_empty() {
            if app.chats.0.is_empty() {
                app.chats.0.insert(Id::new(), SavedChat::default());
            }
            app.regenerate_side_chats();
            if let Some(saved) = app.chats.0.iter().last() {
                let first = (
                    Id::new(),
                    Chats::new(
                        vec![models.first().unwrap().clone()],
                        saved.0.clone(),
                        saved.1.to_mk(),
                    ),
                );

                app.panes = Panes::new(panes::Pane::Chat(first.0.clone()));
                app.panes.last_chat = first.0.clone();
                app.main_view.add_to_chats(first.0, first.1);
            }

            app.options
                .get_create_model_options_index(models.first().unwrap().clone());
        }

        (app, Task::none())
    }

    fn title(&self) -> String {
        String::from("Creative Chat")
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
                self.main_view.remove_chat_stream_by_id(&id);
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
            Message::Generated(id, x, model) => {
                if let Ok(x) = x {
                    let mut mk = Vec::new();
                    let mut is_multi = false;
                    if let Some(chat) = self.chats.0.get_mut(&id) {
                        if let Some(parent) = chat.chats.get_last_parent_mut() {
                            is_multi = !(parent.chat.role() == &chats::chat::Role::User
                                && model.is_none());
                            if !is_multi {
                                let index = parent
                                    .selected_child_index
                                    .unwrap_or(parent.children.len() - 1);
                                parent.children[index]
                                    .chat
                                    .add_to_content(x.content.as_str());
                            } else {
                                for child in parent.children.iter_mut() {
                                    if child.reason == None {
                                        child.reason = Some(Reason::Sibling);
                                    }
                                }
                                parent.add_chat(
                                    ChatBuilder::default()
                                        .role(chats::chat::Role::AI)
                                        .content(x.content.clone())
                                        .build()
                                        .unwrap(),
                                    Some(chats::tree::Reason::Model(model.unwrap().clone())),
                                );
                            }
                        }

                        mk = chat.to_mk();
                    }

                    self.chats.save(CHATS_FILE);
                    self.regenerate_side_chats();

                    self.main_view.update_chats(|chats| {
                        chats
                            .iter_mut()
                            .filter(|chat| chat.1.saved_chat() == &id)
                            .for_each(|(_, chat)| {
                                chat.set_markdown(mk.clone());
                                chat.set_content(text_editor::Content::new());
                                chat.set_images(Vec::new());
                                if !is_multi {
                                    chat.set_state(chats::view::State::Idle);
                                }
                            });
                    });
                }

                Task::none()
            }
            Message::Pull(x) => {
                self.main_view
                    .add_download(Id::new(), Download::new(x.clone()));
                Task::none()
            }

            Message::Generating((id, progress)) => {
                self.main_view.update_chat_streams(|streams| {
                    if let Some(chat) = streams.get_mut(&id) {
                        chat.progress(progress.clone());
                    }
                });

                if let Ok(ChatProgress::Generating(progress)) = progress {
                    let mut mk = Chat::generate_mk(progress.content.as_str());

                    if let Some(chat) = self.chats.0.get_mut(&id) {
                        if let Some(parent) = chat.chats.get_last_parent_mut() {
                            if parent.chat.role() == &chats::chat::Role::User {
                                let index = parent
                                    .selected_child_index
                                    .unwrap_or(parent.children.len() - 1);
                                parent.children[index]
                                    .chat
                                    .add_to_content(progress.content.as_str());
                                mk = Chat::generate_mk(&parent.children[index].chat.content());
                            }
                        }
                    }

                    self.main_view.update_chats(|chats| {
                        chats
                            .iter_mut()
                            .filter(|chat| chat.1.saved_chat() == &id)
                            .for_each(|(_, chat)| {
                                chat.update_markdown(|x| {
                                    x.remove(x.len() - 1);
                                });
                                chat.set_state(chats::view::State::Generating);
                                chat.add_markdown(mk.clone());
                            });
                    });
                } else if let Ok(ChatProgress::Finished) = progress {
                    self.chats.save(CHATS_FILE);
                    self.regenerate_side_chats();

                    self.main_view.update_chats(|chats| {
                        chats
                            .iter_mut()
                            .filter(|chat| chat.1.saved_chat() == &id)
                            .for_each(|(_, chat)| {
                                chat.set_content(text_editor::Content::new());
                                chat.set_images(Vec::new());
                                chat.set_state(chats::view::State::Idle);
                            });
                    });
                }

                Task::none()
            }
            Message::Pulling((id, progress)) => {
                if let Ok(progress) = progress.clone() {
                    if let DownloadProgress::Finished = progress {
                        self.main_view.remove_download_by_id(&id);
                        let models = self.logic.get_models();
                        self.logic.combo_models = combo_box::State::new(models.clone());
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
