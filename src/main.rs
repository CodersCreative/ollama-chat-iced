pub mod save;
pub mod utils;
pub mod chat;
pub mod sidebar;
pub mod view;
pub mod helper;
pub mod update;
pub mod style;
pub mod start;

use crate::{
    save::Save,
    sidebar::chats::Chats as SideChats,
    chat::get_models
};

use iced::{
    clipboard, widget::{combo_box, container, markdown, row}, Element, Task, Theme
};

use update::Logic;
use sidebar::SideBarState;
use std::sync::Arc;
use crate::view::View;

const SAVE_FILE: &str = "chat.json";
const PREVIEW_LEN: usize = 20;

fn main() -> iced::Result{
    iced::application(ChatApp::title, ChatApp::update, ChatApp::view).theme(ChatApp::theme).run()
}

pub struct ChatApp{
    pub save: Save,
    pub main_view: View,
    pub markdown: Vec<Vec<markdown::Item>>,
    pub logic : Logic,
}

#[derive(Debug, Clone)]
pub enum Message{
    Edit(String),
    ChangeTheme(Theme),
    ChangeIndent(String),
    SaveToClipboard(String),
    ChangeModel(String),
    ChangeStart(String),
    ChangeChat(usize),
    RemoveChat(usize),
    Received(Result<String, String>),
    URLClicked(markdown::Url),
    ShowSettings,
    SideBar,
    Submit,
    NewChat,
}

impl Default for ChatApp{
    fn default() -> Self {
        let (app, _) = Self::init();
        app
    }
}

impl ChatApp{
    fn get_model(&self)-> String{
        match self.save.ai_model.clone().is_empty(){true => "qwen:0.5b".to_string(), _ => self.save.ai_model.clone()}
    }

    fn new() -> Self{
        Self{
            save: Save::new(String::new()),
            main_view: View::new(),
            logic: Logic::new(),
            markdown: Vec::new(),
        }
    }

    fn new_with_save(save : Save) -> Self{
        Self{
            save,
            main_view: View::new(),
            logic: Logic::new(),
            markdown: Vec::new(),
        }
    }

    fn init() -> (ChatApp, Task<Message>){
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();

        let mut app = Self::new_with_save(match Save::load(SAVE_FILE){
            Ok(x) => x,
            Err(_) => Save::new(String::new()),
        });

        let models = tokio_runtime.block_on(get_models(Arc::clone(&app.logic.ollama)));
        app.logic.models = combo_box::State::new(models.clone());
        app.main_view.chats = SideChats::new(app.save.get_chat_previews());

        if let Some(i) = app.save.theme{
            app.main_view.theme = Theme::ALL[i].clone()
        }

        app.main_view.indent = app.save.code_indent.to_string();

        if app.save.ai_model.is_empty() && !models.is_empty(){
            app.save.ai_model = models[0].clone();
        }

        if let Some(chat) = app.save.get_current_chat(){
            let ollama = Arc::clone(&app.logic.ollama);
            app.markdown = chat.to_mk();
            chat::ollama_from_chat(ollama, chat);
        }

        app.logic.chat = app.save.get_current_chat_num();
        
        (app, Task::none())
    }

    fn title(&self) -> String{
        String::from("Creative Chat")
    }

    fn update(&mut self, message : Message) -> Task<Message>{
        match message {
            Message::SaveToClipboard(x) => {
                println!("Save Clip {}", x);
                clipboard::write::<Message>(x.clone())
            },
            Message::ShowSettings => {
                self.main_view.side = match self.main_view.side{
                    SideBarState::Settings => SideBarState::Shown,
                    _ => SideBarState::Settings,
                };
                Task::none()
            },
            Message::URLClicked(x) => {
                open::that_in_background(x.to_string()); 
                Task::none()
            },
            Message::SideBar => {
                self.main_view.side = match self.main_view.side{
                    SideBarState::Hidden => SideBarState::Shown,
                    _ => SideBarState::Hidden,
                };
                Task::none()
            },
            Message::Received(x) => {
                if let Ok(x) = x{
                    return self.received(x.clone());
                }
                Task::none()
            },
            Message::ChangeModel(x) => {
                self.save.set_model(x.clone());
                self.save.save(SAVE_FILE);
                Task::none()
            },
            Message::ChangeStart(x) => {
                self.main_view.start = x;
                Task::none()
            },
            Message::ChangeChat(x) => {
                self.change_chat(x)
            },
            Message::NewChat => {
                self.new_chat()
            },
            Message::RemoveChat(x) => {
                self.remove_chat(x)
            },
            Message::Edit(x) => {
                self.main_view.input = x.clone();
                Task::none()
            },
            Message::Submit => {
                self.submit()
            },
            Message::ChangeTheme(x) => {
                self.change_theme(x.clone())
            },
            Message::ChangeIndent(x) => {
                self.main_view.indent = x.clone();
                if let Ok(size) = x.trim().parse::<usize>(){
                    self.save.code_indent = size;
                }
                Task::none()
            }
        }
    }

    fn view<'a>(&'a self) -> Element<Message>{       
        container(row![
            self.main_view.side_bar(self),
            self.main_view.chat_view(self),
        ]).into()
    }

    fn theme(&self) -> iced::Theme {
        self.main_view.theme.clone()
    }
}

