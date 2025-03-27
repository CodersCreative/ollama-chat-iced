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

use crate::{
    save::Save,
    sidebar::chats::Chats as SideChats,
    options::Options,
    chat::get_models
};
use iced::{
    clipboard, widget::{combo_box, container, row, markdown}, Element, Font, Task, Theme
};

use ollama_rs::generation::chat::ChatMessage;
use options::OptionKey;
use update::Logic;
use sidebar::SideBarState;
use std::{path::PathBuf, sync::Arc};
use crate::view::View;

pub const FONT: &[u8] = include_bytes!("../assets/RobotoMonoNerdFont-Regular.ttf");
const SAVE_FILE: &str = "chat.json";
const SETTINGS_FILE: &str = "settings.json";
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
    pub options : Options,
    pub markdown: Vec<Vec<markdown::Item>>,
    pub logic : Logic,
}

#[derive(Debug, Clone)]
pub enum Message{
    Edit(String),
    ChangeTheme(Theme),
    ChangeIndent(String),
    SaveToClipboard(String),
    Regenerate,
    ChangeModel(String),
    ChangeStart(String),
    ChangeChat(usize),
    RemoveChat(usize),
    Received(Result<ChatMessage, String>),
    PickedImage(Result<Vec<PathBuf>, String>),
    ChangeOptionNum((String, OptionKey)),
    SubmitOptionNum(OptionKey),
    ChangeOptionBool((bool, OptionKey)),
    ClickedOption(OptionKey),
    ResetOption(OptionKey),
    PickImage,
    RemoveImage(PathBuf),
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
        Self::new_with_save(Save::new(String::new()))
    }

    fn new_with_save(save : Save) -> Self{
        Self{
            save,
            main_view: View::new(),
            logic: Logic::new(),
            markdown: Vec::new(),
            options: Options::default(),
        }
    }

    fn init() -> (ChatApp, Task<Message>){
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();

        let mut app = Self::new_with_save(match Save::load(SAVE_FILE){
            Ok(x) => x,
            Err(_) => Save::new(String::new()),
        });

        app.options = match Options::load(SETTINGS_FILE) {
            Ok(x) => x,
            Err(_) => Options::default(),
        };

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
            app.markdown = chat.to_mk();
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
            Message::Regenerate => {
                if let Some(i) = self.save.get_index(self.save.last){
                    self.save.chats[i].0.pop();
                    return self.submit(false);
                }

                Task::none()
            },
            Message::ChangeOptionBool(x) => {
                let m_index = self.options.get_create_model_options_index(self.get_model());
                let index = self.options.0[m_index].get_key_index(x.1);
                self.options.0[m_index].0[index].bool_value = x.0;
                self.options.save(SETTINGS_FILE);
                Task::none()
            },
            Message::ChangeOptionNum(x) => {
                let m_index = self.options.get_create_model_options_index(self.get_model());
                let index = self.options.0[m_index].get_key_index(x.1);
                self.options.0[m_index].0[index].temp = x.0;
                Task::none()
            },
            Message::SubmitOptionNum(x) => {
                let m_index = self.options.get_create_model_options_index(self.get_model());
                let index = self.options.0[m_index].get_key_index(x);
                if let Ok(num) = self.options.0[m_index].0[index].temp.parse::<f32>(){
                    let mut value = self.options.0[m_index].0[index].num_value.unwrap();
                    value.0 = num;
                    self.options.0[m_index].0[index].num_value = Some(value);
                    self.options.save(SETTINGS_FILE);
                }else{
                    self.options.0[m_index].0[index].temp = self.options.0[m_index].0[index].num_value.unwrap().0.to_string();
                }
                Task::none()
            },
            Message::ResetOption(x) => {
                let m_index = self.options.get_create_model_options_index(self.get_model());
                let index = self.options.0[m_index].get_key_index(x);
                let mut value = self.options.0[m_index].0[index].num_value.unwrap();
                value.0 = value.1;
                self.options.0[m_index].0[index].num_value = Some(value);
                self.options.0[m_index].0[index].temp = value.1.to_string();
                self.options.0[m_index].0[index].bool_value = false;
                self.options.save(SETTINGS_FILE);
                Task::none()
            },
            Message::ClickedOption(x) => {
                let m_index = self.options.get_create_model_options_index(self.get_model());
                let index = self.options.0[m_index].get_key_index(x);
                self.options.0[m_index].0[index].shown = !self.options.0[m_index].0[index].shown;
                Task::none()
            },
            Message::ShowSettings => {
                self.main_view.side = match self.main_view.side{
                    SideBarState::Settings => SideBarState::Shown,
                    _ => {
                        let _ = self.options.get_create_model_options_index(self.get_model());
                        SideBarState::Settings
                    },
                };
                Task::none()
            },
            Message::URLClicked(x) => {
                open::that_in_background(x.to_string()); 
                Task::none()
            },
            Message::RemoveImage(x) => {
                if let Ok(x) = self.main_view.images.binary_search(&x){
                    self.main_view.images.remove(x);
                }
                Task::none()
            },
            Message::SideBar => {
                self.main_view.side = match self.main_view.side{
                    SideBarState::Hidden => SideBarState::Shown,
                    _ => SideBarState::Hidden,
                };
                Task::none()
            },
            Message::PickedImage(x) => {
                if let Ok(mut x) = x{
                    self.main_view.images.append(&mut x);
                }
                Task::none()
            }
            Message::Received(x) => {
                if let Ok(x) = x{
                    return self.received(x.clone());
                }
                Task::none()
            },
            Message::ChangeModel(x) => {
                self.save.set_model(x.clone());
                self.save.save(SAVE_FILE);
                let _ = self.options.get_create_model_options_index(self.get_model());
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
                self.submit(true)
            },
            Message::PickImage => {
                Self::pick_images()
            }
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

