pub mod save;
pub mod utils;
pub mod chat;
pub mod sidebar;
pub mod view;
pub mod helper;
pub mod update;

use crate::{
    save::Save,
    sidebar::chats::Chats as SideChats,
    chat::get_models
};

use iced::{
    widget::{combo_box, container, row}, 
    Element, Settings, Theme, Application, Command, clipboard
};

use update::Logic;
use std::sync::Arc;
use crate::view::View;

const SAVE_FILE: &str = "chat.json";
const PREVIEW_LEN: usize = 20;

fn main() -> iced::Result{
    ChatApp::run(Settings::default())
}

pub struct ChatApp{
    save: Save,
    main_view: View,
    logic : Logic,
}

#[derive(Debug, Clone)]
pub enum Message{
    Edit(String),
    ChangeTheme(Theme),
    ChangeIndent(String),
    SaveToClipboard(String),
    ChangeModel(String),
    ChangeChat(usize),
    RemoveChat(usize),
    Received(Result<String, String>),
    Submit,
    NewChat,
}

impl ChatApp{
    fn get_model(&self)-> String{
        match self.save.ai_model.clone().is_empty(){true => "qwen:0.5b".to_string(), _ => self.save.ai_model.clone()}
    }

    fn new_plain() -> Self{
        Self{
            save: Save::new(String::new()),
            main_view: View::new(),
            logic: Logic::new(),
        }
    }
}

impl Application for ChatApp{
    type Message = Message;
    type Executor = iced::executor::Default;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (ChatApp, Command<Message>){
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();

        let mut app = Self::new_plain();
        
        match Save::load(SAVE_FILE){
            Ok(x) => {
                app.save = x;
            },
            Err(..) => {}
        };

        let ollama = Arc::clone(&app.logic.ollama);
        let models = tokio_runtime.block_on(get_models(ollama));
        app.logic.models = combo_box::State::new(models.clone());
        app.main_view.chats = SideChats::new(app.save.get_chat_previews());

        if let Some(i) = app.save.theme{
            app.main_view.theme = Theme::ALL[i].clone()
        }

        app.main_view.indent = app.save.code_indent.to_string();

        if app.save.ai_model.is_empty(){
            app.save.ai_model = models[0].clone();
        }

        let chat = app.save.get_current_chat();
        if let Some(chat) = chat{
            let ollama = Arc::clone(&app.logic.ollama);
            save::Save::ollama_from_chat(ollama, chat);
        }

        app.logic.chat = app.save.get_current_chat_num();
        
        (app, Command::none())
    }

    fn title(&self) -> String{
        String::from("Creative Chat")
    }

    fn update(&mut self, message : Message) -> Command<Message>{
        match message {
            Message::SaveToClipboard(x) => {
                println!("Save Clip {}", x);
                clipboard::write::<Message>(x.clone())
            },
            Message::Received(x) => {
                if let Ok(x) = x{
                    return self.received(x.clone());
                }
                Command::none()
            },
            Message::ChangeModel(x) => {
                self.save.set_model(x.clone());
                self.save.save(SAVE_FILE);
                Command::none()
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
                Command::none()
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
                Command::none()
            }
        }
    }

    fn view<'a>(&'a self) -> Element<'_, Message>{       
        container(row![
            self.main_view.chat_side_bar(self),
            self.main_view.chat_view(self),
        ]).into()
    }

    fn theme(&self) -> iced::Theme {
        self.main_view.theme.clone()
    }
}

