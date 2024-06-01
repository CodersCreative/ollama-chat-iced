use chat::{run_ollama, get_models};
use iced::widget::combo_box::State;
use iced::widget::{button, column, combo_box, container, keyed_column, mouse_area, row, scrollable, text, text_input};
use iced::{Alignment, Length};
use cli_clipboard;
use iced::{Element, Settings, Theme, Application, Command, Subscription, subscription,clipboard};
use iced_futures::core::{Widget};
use ollama_rs::Ollama;
use save::{Chat, Chats};
use tokio::sync::Mutex as TMutex;
use utils::darken_colour;
pub mod save;
pub mod utils;
pub mod chat;
pub mod chats;
pub mod sidebar;
use crate::save::Save;
use crate::chats::SideChats;
use std::sync::Arc;

const SAVE_FILE: &str = "chat.json";
static THEME: Theme = Theme::CatppuccinMocha; //CatppuccinMocha
const PREVIEW_LEN: usize = 20;

fn main() -> iced::Result{
    ChatApp::run(Settings::default())
}


struct ChatApp{
    input: String,
    save: Save,
    models: combo_box::State<String>,
    theme: combo_box::State<String>,
    chats: SideChats,
    chat: Option<usize>,
    ollama: Arc<TMutex<Ollama>>,
}

#[derive(Debug, Clone)]
enum Message{
    Edit(String),
    ChangeModel(String),
    ChangeTheme(String),
    SaveToClipboard(String),
    ChangeChat(usize),
    RemoveChat(usize),
    NewChat,
    Submit,
}

impl ChatApp{
    fn get_model(&self)-> String{
        match self.save.ai_model.clone().is_empty(){true => "qwen:0.5b".to_string(), _ => self.save.ai_model.clone()}
    }

    fn new_plain() -> Self{
        Self{
            input: String::new(),
            save: Save::new(String::new(), String::new()),
            models: combo_box::State::new(Vec::new()),
            theme: combo_box::State::new(Vec::new()),
            chats: SideChats::new(Vec::new()),
            chat: None,
            ollama: Arc::new(TMutex::new(chat::get_model())),
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

        let mut app = (Self::new_plain(), Command::none());
        
        match Save::load(SAVE_FILE){
            Ok(x) => {
                app.0.save = x;
            },
            Err(..) => {}
        };

        let ollama = Arc::clone(&app.0.ollama);
        let models = tokio_runtime.block_on(get_models(ollama));
        app.0.models = combo_box::State::new(models.clone());
        app.0.chats = SideChats::new(app.0.save.get_chat_previews());
        if app.0.save.ai_model.is_empty(){
            app.0.save.ai_model = models[0].clone();
        }

        let chat = app.0.save.get_current_chat();
        if let Some(chat) = chat{
            let ollama = Arc::clone(&app.0.ollama);
            save::Save::ollama_from_chat(ollama, chat);
        }
        app.0.chat = app.0.save.get_current_chat_num();
        app
    }

    fn title(&self) -> String{
        String::from("Creative Chat")
    }

    fn update(&mut self, message : Message) -> Command<Message>{
        match message {
            Message::Edit(x) => {
                self.input = x;
            },
            Message::Submit => {
                let ollama = Arc::clone(&self.ollama);
                let input = self.input.clone();
                let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
                let result = tokio_runtime.block_on(run_ollama(input, ollama, self.get_model()));
                if let Ok(result) = result{
                    let index = self.save.get_index(self.save.last);

                    let chats = vec![Chat{
                        name : "User".to_owned(),
                        message : self.input.clone(),
                    },
                    Chat{
                        name : "AI".to_owned(),
                        message : result.trim().to_string(),
                    }];

                    match index{
                        Some(x) => self.save.chats[x].0.extend(chats),
                        None => self.save.chats.push(Chats::new_with_chats(chats)),
                    }
                }
                self.input = String::new();
                self.save.save(SAVE_FILE);
                self.chats = SideChats::new(self.save.get_chat_previews());
            },
            Message::ChangeModel(x) => {
                self.save.set_model(x);
                self.save.save(SAVE_FILE);
            },
            Message::ChangeChat(x) => {
                self.save.last = self.save.chats[x].1;
                self.chat = Some(x);
                let chat = self.save.get_current_chat();
                if let Some(chat) = chat{
                    let ollama = Arc::clone(&self.ollama);
                    save::Save::ollama_from_chat(ollama, chat);
                }
                self.save.save(SAVE_FILE);
            },
            Message::NewChat => {
                let chat = Chats::new();
                self.save.chats.push(chat.clone());
                self.save.last = chat.1.clone();
                self.ollama = Arc::new(TMutex::new(chat::get_model()));
                self.chat = Some(self.save.chats.len() - 1);
                self.chats = SideChats::new(self.save.get_chat_previews());
            },
            Message::RemoveChat(x) => {
                self.save.chats.remove(x);
                self.save.last = self.save.chats.last().unwrap().1.clone();
                self.ollama = Arc::new(TMutex::new(chat::get_model()));
                self.chat = Some(self.save.chats.len() - 1);
                self.chats = SideChats::new(self.save.get_chat_previews());
            },
            Message::ChangeTheme(x) => {
                //self.save.set_model(x);
                //self.save.save(SAVE_FILE);
            },
            Message::SaveToClipboard(x) => {
                println!("Save Clip {}", x);
                return clipboard::write::<Message>(x.clone());
                //cli_clipboard::set_contents(x).unwrap();
                //println!("{}",cli_clipboard::get_contents().unwrap());
            }
        }

        return Command::none();
    }

    fn view(&self) -> Element<'_, Message>{
        let input = text_input("Enter your message", &self.input).on_input(Message::Edit).on_submit(Message::Submit).width(Length::FillPortion(19));
        let chat_view = container(scrollable(self.save.view_chat()).width(Length::Fill)).width(Length::Fill).height(Length::Fill).padding(20);
        let ai_box = container(combo_box(&self.models, self.save.ai_model.as_str(), None, Message::ChangeModel));
        //let theme_box = container(combo_box(&self.theme, self.save.ai_model.as_str(), None, Message::ChangeModel));
        let header = container(text("Chats").size(36)).style(container::Appearance{
            background : Some(iced::Background::Color(THEME.palette().text)),
            text_color : Some(THEME.palette().background),
            ..Default::default()
        }).width(Length::Fill);
        let new_text = text("+").horizontal_alignment(iced::alignment::Horizontal::Center).width(Length::Fill).size(25);
        let new_button = container(mouse_area(new_text).on_press(Message::NewChat)).width(Length::Fill).style(container::Appearance{
            background : Some(iced::Background::Color(self.theme().palette().primary)),
            text_color: Some(THEME.palette().background),
            ..Default::default()
        });
        let bottom = container(column![
            ai_box,
            new_button,
        ]);
        let chat_chooser = container(self.chats.view(self.chat)).height(Length::Fill);
        let chat = container(column![
            chat_view,
            input,
        ]).width(Length::FillPortion(5));
        let side = container(column![
            header,
            chat_chooser,
            bottom,
        ]).width(Length::FillPortion(1)).style(container::Appearance{
            background : Some(iced::Background::Color(darken_colour(THEME.palette().background.clone(), 0.01))),
            ..Default::default()
        }); 
        container(row![
            side,
            chat,
        ]).into()
    }

    fn theme(&self) -> iced::Theme {
        THEME.clone()
    }
}

