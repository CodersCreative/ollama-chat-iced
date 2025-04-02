pub mod save;
pub mod utils;
pub mod chat;
pub mod sidebar;
pub mod view;
pub mod helper;
pub mod update;
pub mod style;
pub mod start;
pub mod options;
pub mod panes;
pub mod models;
pub mod download;
pub mod sound;

use crate::{
    save::Save,
    sidebar::chats::Chats as SideChats,
};
use chat::ChatProgress;
use download::{Download, DownloadProgress};
use iced::{
    clipboard, widget::{combo_box, container, markdown, row, text_editor}, Element, Font, Subscription, Task, Theme
};

use models::{Models, ModelsMessage, SavedModels};
use natural_tts::{models::{gtts::GttsModel, tts_rs::TtsModel}, NaturalTts, NaturalTtsBuilder};
use options::{Options, OptionMessage, SavedOptions};
use panes::Panes;
use save::{chat::{Chat, Role}, chats::{ChatsMessage, SavedChats}};
use sound::transcribe;
use update::Logic;
use sidebar::SideBarState;
use panes::PaneMessage;
use crate::view::View;

pub const FONT: &[u8] = include_bytes!("../assets/RobotoMonoNerdFont-Regular.ttf");
const SAVE_FILE: &str = "chat.json";
const PREVIEW_LEN: usize = 25;

fn main() -> iced::Result{
    //let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    //
    //let resp = tokio_runtime.block_on(transcribe()).unwrap();
    //println!("{}", resp);
    let font = Font{
        family: iced::font::Family::Name("Roboto"),
        style: iced::font::Style::Normal,
        stretch: iced::font::Stretch::Normal,
        weight: iced::font::Weight::Normal,
    };
    iced::application(ChatApp::title, ChatApp::update, ChatApp::view).theme(ChatApp::theme).font(FONT).default_font(font).subscription(ChatApp::subscription).run()
}

pub struct ChatApp{
    pub save: Save,
    pub main_view: View,
    pub options : SavedOptions,
    pub model_info : SavedModels, 
    pub logic : Logic,
    pub panes : Panes,
    pub tts : NaturalTts,
}

#[derive(Debug, Clone)]
pub enum Message{
    Pane(PaneMessage),
    Models(ModelsMessage, i32),
    Option(OptionMessage, i32),
    Chats(ChatsMessage, i32),
    ChangeTheme(Theme),
    SaveToClipboard(String),
    RemoveChat(usize),
    URLClicked(markdown::Url),
    ShowSettings,
    SideBar,
    Pulling((usize, Result<DownloadProgress, String>)),
    Generating((i32, Result<ChatProgress, String>)),
    StopGenerating(i32),
    Pull(String),
    StopPull(usize),
    None,
}

impl Default for ChatApp{
    fn default() -> Self {
        let (app, _) = Self::init();
        app
    }
}

impl ChatApp{
    fn new() -> Self{
        Self::new_with_save(Save::new())
    }

    fn new_with_save(save : Save) -> Self{
        Self{
            save,
            panes: Panes::new(panes::Pane::Chat(0)),
            main_view: View::new(),
            logic: Logic::new(),
            model_info : SavedModels::init().unwrap(),
            options: SavedOptions::default(),
            tts : NaturalTtsBuilder::default().default_model(natural_tts::Model::Gtts).gtts_model(GttsModel::default()).tts_model(TtsModel::default()).build().unwrap(),
        }
    }

    fn init() -> (ChatApp, Task<Message>){
        let mut app = Self::new_with_save(match Save::load(SAVE_FILE){
            Ok(x) => x,
            Err(_) => Save::new(),
        });

        app.options = match SavedOptions::load(options::SETTINGS_FILE) {
            Ok(x) => x,
            Err(_) => SavedOptions::default(),
        };

        let models = app.logic.get_models();
        app.logic.combo_models = combo_box::State::new(models.clone());


        if let Some(i) = app.save.theme{
            app.main_view.theme = Theme::ALL[i].clone()
        }


        if !models.is_empty(){
            if app.save.chats.is_empty(){
                app.save.chats.push(SavedChats::new());
            }else{
                app.save.chats.iter_mut().for_each(|x| {
                    if let Some(y) = x.0.last(){
                        if y.role != Role::AI{
                            x.0.remove(x.0.len() - 1);
                        }
                    } 
                });
            }

            app.main_view.side_chats = SideChats::new(app.save.get_chat_previews());
            let saved = app.save.chats.last().unwrap();
            let first = save::chats::Chats::new(models.first().unwrap().clone(), saved.1, saved.to_mk());

            app.panes = Panes::new(panes::Pane::Chat(first.id));
            app.panes.last_chat = first.id;
            app.main_view.chats.push(first);
            app.options.get_create_model_options_index(models.first().unwrap().clone());
        }
        app.logic.chat = app.save.get_current_chat_num();
        
        (app, Task::none())
    }

    fn title(&self) -> String{
        String::from("Creative Chat")
    }

    fn update(&mut self, message : Message) -> Task<Message>{
        match message {
            Message::Option(x, i) => {
                x.handle(Options::get_from_id(self, i).clone(), self)
            },
            Message::Models(x, i) => {
                x.handle(Models::get_from_id(self, i).clone(), self)
            },
            Message::None => Task::none(),
            Message::SaveToClipboard(x) => {
                println!("Save Clip {}", x);
                clipboard::write::<Message>(x.clone())
            },
            Message::Chats(x, i) => {
                x.handle(i, self)
            }
            Message::Pane(x) => {
                x.handle(self)
            }
            Message::URLClicked(x) => {
                open::that_in_background(x.to_string()); 
                Task::none()
            },
            Message::StopPull(id) => {
                for (i, x) in self.main_view.downloads.iter().enumerate(){
                    if x.id == id{
                        self.main_view.downloads.remove(i);
                        break;
                    }
                }
                Task::none()
            },
            Message::StopGenerating(id) => {
                for (i, x) in self.main_view.chat_streams.iter().enumerate(){
                    if x.id == id{
                        self.main_view.chat_streams.remove(i);
                        break;
                    }
                }
                Task::none()
            },
            Message::ShowSettings => {
                self.main_view.side = match self.main_view.side{
                    SideBarState::Settings => SideBarState::Shown,
                    _ => SideBarState::Settings,
                };
                Task::none()
            },
            Message::SideBar => {
                self.main_view.side = match self.main_view.side{
                    SideBarState::Hidden => SideBarState::Shown,
                    _ => SideBarState::Hidden,
                };
                Task::none()
            },
            Message::RemoveChat(x) => {
                return self.remove_chat(x)
            },
            Message::ChangeTheme(x) => {
                self.change_theme(x.clone())
            },
            Message::Pull(x) => {
                self.main_view.downloads.push(Download::new(self.main_view.id, x.clone()));
                self.main_view.id += 1;
                Task::none()
            },
            Message::Generating((id, progress)) => {
                if let Some(chat) = self.main_view.chat_streams.iter_mut().find(|chat| chat.id == id){
                    chat.progress(progress.clone());
                }

                if let Ok(ChatProgress::Generating(progress)) = progress{
                    let mut mk = Chat::generate_mk(progress.content.as_str());
                    let mut first = true;
                    
                    if let Some(chat) = self.save.chats.iter_mut().find(|chat| chat.1 == id){
                        let index = chat.0.len() - 1;
                        if chat.0.last().unwrap().role == save::chat::Role::AI{
                            chat.0[index].message.push_str(progress.content.as_str());
                            mk = Chat::generate_mk(&chat.0[index].message);
                            first = false;
                        }else{
                            chat.0.push(Chat::new(&save::chat::Role::AI, &progress.content, Vec::new()));
                            first = true;
                        }
                    }
                    
                    self.main_view.chats.iter_mut().filter(|chat| chat.saved_id == id).for_each(|chat|{
                        if !first{
                            chat.markdown.remove(chat.markdown.len() - 1);
                        }
                        chat.state = save::chats::State::Generating;
                        chat.markdown.push(mk.clone());
                    });
                }else if let Ok(ChatProgress::Finished) = progress {
                    self.save.save(SAVE_FILE);
                    self.main_view.side_chats = SideChats::new(self.save.get_chat_previews());

                    self.main_view.chats.iter_mut().filter(|chat| chat.saved_id == id).for_each(|chat|{
                        chat.input = text_editor::Content::new();
                        chat.images = Vec::new();
                        chat.state = save::chats::State::Idle;
                    });
                }
                
                Task::none()
            }
            Message::Pulling((id, progress)) => {
                if let Some(download) = self.main_view.downloads.iter_mut().find(|download| download.id == id){
                    download.progress(progress);
                }
                Task::none()
            }
        }
    }

    fn view<'a>(&'a self) -> Element<'a, Message>{
        container(row![
            self.main_view.side_bar(self),
            self.panes.view(self),
            //self.main_view.chat_view(self),
        ]).into()
    }

    fn theme(&self) -> iced::Theme {
        self.main_view.theme.clone()
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut actions : Vec<Subscription<Message>>  = self.main_view.downloads.iter().map(|x| x.subscription(self)).collect();
        self.main_view.chat_streams.iter().for_each(|x| actions.push(x.subscription(self)));
        Subscription::batch(actions)
    }
}

