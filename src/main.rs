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

use crate::{
    save::Save,
    sidebar::chats::Chats as SideChats,
};
use iced::{
    clipboard, widget::{combo_box, container, markdown, row}, Element, Font, Task, Theme
};

use models::{Models, ModelsMessage, SavedModels};
use options::{Options, OptionMessage, SavedOptions};
use panes::Panes;
use save::chats::{ChatsMessage, SavedChats};
use update::Logic;
use sidebar::SideBarState;
use panes::PaneMessage;
use crate::view::View;

pub const FONT: &[u8] = include_bytes!("../assets/RobotoMonoNerdFont-Regular.ttf");
const SAVE_FILE: &str = "chat.json";
const PREVIEW_LEN: usize = 20;

fn main() -> iced::Result{
    let font = Font{
        family: iced::font::Family::Name("Roboto"),
        style: iced::font::Style::Normal,
        stretch: iced::font::Stretch::Normal,
        weight: iced::font::Weight::Normal,
    };
    iced::application(ChatApp::title, ChatApp::update, ChatApp::view).theme(ChatApp::theme).font(FONT).default_font(font).run()
}

pub struct ChatApp{
    pub save: Save,
    pub main_view: View,
    pub options : SavedOptions,
    pub model_info : SavedModels, 
    pub logic : Logic,
    pub panes : Panes
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
        app.main_view.side_chats = SideChats::new(app.save.get_chat_previews());

        if let Some(i) = app.save.theme{
            app.main_view.theme = Theme::ALL[i].clone()
        }


        if !models.is_empty(){
            if app.save.chats.is_empty(){
                app.save.chats.push(SavedChats::new());
            }

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
            Message::SaveToClipboard(x) => {
                println!("Save Clip {}", x);
                clipboard::write::<Message>(x.clone())
            },
            Message::Chats(x, i) => {
                x.handle(save::chats::Chats::get_from_id(self, i).clone(), self)
            }
            Message::Pane(x) => {
                x.handle(self)
            }
            Message::URLClicked(x) => {
                open::that_in_background(x.to_string()); 
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
}

