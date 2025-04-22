pub mod call;
pub mod chats;
pub mod common;
pub mod download;
pub mod helper;
pub mod llm;
pub mod models;
pub mod options;
pub mod panes;
pub mod save;
pub mod sidebar;
pub mod sound;
pub mod start;
pub mod style;
pub mod update;
pub mod utils;
pub mod view;
// pub mod database;

use crate::{save::Save, sidebar::chats::Chats as SideChats};
use call::{Call, CallMessage};
use common::Id;
use download::{Download, DownloadProgress};
use iced::{
    clipboard, event,
    widget::{combo_box, container, markdown, row, text_editor},
    window, Element, Event, Font, Subscription, Task, Theme,
};
use llm::ChatProgress;
use models::{Models, ModelsMessage, SavedModels};
use natural_tts::{
    models::{gtts::GttsModel, tts_rs::TtsModel},
    NaturalTts, NaturalTtsBuilder,
};
use ollama_rs::generation::chat::ChatMessage;
use options::{OptionMessage, Options, SavedOptions};
use panes::PaneMessage;
use panes::Panes;
use save::{
    chat::{Chat, Role},
    chats::{ChatsMessage, SavedChats},
};
use sidebar::SideBarState;
use update::Logic;
use view::View;

pub const FONT: &[u8] = include_bytes!("../assets/RobotoMonoNerdFont-Regular.ttf");
const SAVE_FILE: &str = "chat.json";
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
    pub logic: Logic,
    pub panes: Panes,
    pub tts: NaturalTts,
    pub call: Call,
    pub potrait: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Pane(PaneMessage),
    Call(CallMessage),
    Models(ModelsMessage, Id),
    Option(OptionMessage, Id),
    Chats(ChatsMessage, Id),
    ChangeTheme(Theme),
    SaveToClipboard(String),
    RemoveChat(usize),
    URLClicked(markdown::Url),
    ChangeUsePanels(bool),
    ShowSettings,
    SideBar,
    Pulling((usize, Result<DownloadProgress, String>)),
    Generating((Id, Result<ChatProgress, String>)),
    Generated(Id, Result<ChatMessage, String>),
    StopGenerating(Id),
    Pull(String),
    StopPull(usize),
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
        Self::new_with_save(Save::new())
    }

    fn new_with_save(save: Save) -> Self {
        Self {
            save,
            panes: Panes::new(panes::Pane::Chat(Id::new())),
            main_view: View::new(),
            logic: Logic::new(),
            model_info: SavedModels::init().unwrap(),
            options: SavedOptions::default(),
            tts: NaturalTtsBuilder::default()
                .default_model(natural_tts::Model::Gtts)
                .gtts_model(GttsModel::default())
                .tts_model(TtsModel::default())
                .build()
                .unwrap(),
            call: Call::new("".to_string()),
            potrait: false,
        }
    }

    fn init() -> (ChatApp, Task<Message>) {
        let mut app = Self::new_with_save(match Save::load(SAVE_FILE) {
            Ok(x) => x,
            Err(_) => Save::new(),
        });

        app.options = match SavedOptions::load(options::SETTINGS_FILE) {
            Ok(x) => x,
            Err(_) => SavedOptions::default(),
        };

        let models = app.logic.get_models();
        app.logic.combo_models = combo_box::State::new(models.clone());

        if let Some(i) = app.save.theme {
            app.main_view.set_theme(Theme::ALL[i].clone());
        }

        if !models.is_empty() {
            if app.save.chats.is_empty() {
                app.save.chats.push(SavedChats::new());
            } else {
                app.save.chats.iter_mut().for_each(|x| {
                    if let Some(y) = x.0.last() {
                        if y.role() != &Role::AI {
                            x.0.remove(x.0.len() - 1);
                        }
                    }
                });
            }

            app.main_view
                .set_side_chats(SideChats::new(app.save.get_chat_previews()));
            let saved = app.save.chats.last().unwrap();
            let first = chats::Chats::new(models.first().unwrap().clone(), saved.1, saved.to_mk());

            app.panes = Panes::new(panes::Pane::Chat(first.id().clone()));
            app.panes.last_chat = first.id().clone();
            app.main_view.add_to_chats(first);
            app.options
                .get_create_model_options_index(models.first().unwrap().clone());
        }
        // app.logic.chat = app.save.get_current_chat_num();

        (app, Task::none())
    }

    fn title(&self) -> String {
        String::from("Creative Chat")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Option(x, i) => x.handle(Options::get_from_id(self, i).clone(), self),
            Message::ChangeUsePanels(x) => {
                self.save.use_panes = x;
                self.save.save(SAVE_FILE);
                Task::none()
            }
            Message::Models(x, i) => x.handle(Models::get_from_id(self, i).clone(), self),
            Message::None => Task::none(),
            Message::SaveToClipboard(x) => {
                println!("Save Clip {}", x);
                clipboard::write::<Message>(x.clone())
            }
            Message::Chats(x, i) => x.handle(i, self),
            Message::Pane(x) => x.handle(self),
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
                self.main_view.update_downloads(|downloads| {
                    for (i, x) in downloads.iter().enumerate() {
                        if x.id == id {
                            downloads.remove(i);
                            break;
                        }
                    }
                });
                Task::none()
            }
            Message::StopGenerating(id) => {
                self.main_view.update_chat_streams(|streams| {
                    for (i, x) in streams.iter().enumerate() {
                        if x.id == id {
                            streams.remove(i);
                            break;
                        }
                    }
                });
                Task::none()
            }
            Message::ShowSettings => {
                self.main_view
                    .set_side_state(match self.main_view.side_state() {
                        SideBarState::Settings => SideBarState::Shown,
                        _ => SideBarState::Settings,
                    });
                Task::none()
            }
            Message::SideBar => {
                self.main_view
                    .set_side_state(match self.main_view.side_state() {
                        SideBarState::Hidden => SideBarState::Shown,
                        _ => SideBarState::Hidden,
                    });
                Task::none()
            }
            Message::RemoveChat(x) => return self.remove_chat(x),
            Message::ChangeTheme(x) => self.change_theme(x.clone()),
            Message::Generated(id, x) => {
                if let Ok(x) = x {
                    let mut mk = Chat::generate_mk(x.content.as_str());
                    let mut first = true;
                    if let Some(chat) = self.save.chats.iter_mut().find(|chat| chat.1 == id) {
                        let index = chat.0.len() - 1;
                        if chat.0.last().unwrap().role() == &save::chat::Role::AI {
                            chat.0[index].add_to_content(x.content.as_str());
                            mk = Chat::generate_mk(&chat.0[index].content());
                            first = false;
                        } else {
                            chat.0.push(Chat::new(
                                &save::chat::Role::AI,
                                &x.content,
                                Vec::new(),
                                x.tool_calls,
                            ));
                            first = true;
                        }
                    }

                    self.main_view.update_chats(|chats| {
                        chats
                            .iter_mut()
                            .filter(|chat| chat.saved_chat() == &id)
                            .for_each(|chat| {
                                if !first {
                                    chat.update_markdown(|x| {
                                        x.remove(x.len() - 1);
                                    });
                                    // chat.markdown.remove(chat.markdown.len() - 1);
                                }
                                chat.set_state(chats::State::Generating);
                                chat.add_markdown(mk.clone());
                            });
                    });

                    self.save.save(SAVE_FILE);
                    self.main_view
                        .set_side_chats(SideChats::new(self.save.get_chat_previews()));

                    self.main_view.update_chats(|chats| {
                        chats
                            .iter_mut()
                            .filter(|chat| chat.saved_chat() == &id)
                            .for_each(|chat| {
                                chat.set_content(text_editor::Content::new());
                                chat.set_images(Vec::new());
                                chat.set_state(chats::State::Idle);
                            });
                    });
                }

                Task::none()
            }
            Message::Pull(x) => {
                self.main_view
                    .add_download(Download::new(self.main_view.id().clone(), x.clone()));
                self.main_view.set_id(self.main_view.id() + 1);
                Task::none()
            }

            Message::Generating((id, progress)) => {
                self.main_view.update_chat_streams(|streams| {
                    if let Some(chat) = streams.iter_mut().find(|chat| chat.id == id) {
                        chat.progress(progress.clone());
                    }
                });

                if let Ok(ChatProgress::Generating(progress)) = progress {
                    let mut mk = Chat::generate_mk(progress.content.as_str());
                    let mut first = true;

                    if let Some(chat) = self.save.chats.iter_mut().find(|chat| chat.1 == id) {
                        let index = chat.0.len() - 1;
                        if chat.0.last().unwrap().role() == &save::chat::Role::AI {
                            chat.0[index].add_to_content(progress.content.as_str());
                            mk = Chat::generate_mk(&chat.0[index].content());
                            first = false;
                        } else {
                            chat.0.push(Chat::new(
                                &save::chat::Role::AI,
                                &progress.content,
                                Vec::new(),
                                progress.tool_calls,
                            ));
                            first = true;
                        }
                    }

                    self.main_view.update_chats(|chats| {
                        chats
                            .iter_mut()
                            .filter(|chat| chat.saved_chat() == &id)
                            .for_each(|chat| {
                                if !first {
                                    chat.update_markdown(|x| {
                                        x.remove(x.len() - 1);
                                    });
                                }
                                chat.set_state(chats::State::Generating);
                                chat.add_markdown(mk.clone());
                            });
                    });
                } else if let Ok(ChatProgress::Finished) = progress {
                    self.save.save(SAVE_FILE);
                    self.main_view
                        .set_side_chats(SideChats::new(self.save.get_chat_previews()));

                    self.main_view.update_chats(|chats| {
                        chats
                            .iter_mut()
                            .filter(|chat| chat.saved_chat() == &id)
                            .for_each(|chat| {
                                chat.set_content(text_editor::Content::new());
                                chat.set_images(Vec::new());
                                chat.set_state(chats::State::Idle);
                            });
                    });
                }

                Task::none()
            }
            Message::Pulling((id, progress)) => {
                self.main_view.update_downloads(move |x| {
                    if let Some(download) = x.iter_mut().find(|download| download.id == id) {
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
            .map(|x| x.subscription(self))
            .collect();
        self.main_view
            .chat_streams()
            .iter()
            .for_each(|x| actions.push(x.subscription(self)));
        actions.push(event::listen().map(Message::EventOccurred));
        Subscription::batch(actions)
    }
}
