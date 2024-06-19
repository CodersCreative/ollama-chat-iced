use iced::widget::checkbox::Appearance;
use iced::widget::{column, container, horizontal_space, keyed_column, row, text, mouse_area};
use iced::{Background, Border, Element, Length, Theme};
use iced_futures::core::Widget;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use ollama_rs::Ollama;
use crate::chat::get_model;
use crate::Message;
use rand::Rng;
use crate::utils::get_preview;
use tokio::sync::Mutex;
use std::sync::Arc;
use iced::Color;
use crate::utils::{darken_colour, lighten_colour};

#[derive(Serialize, Deserialize, Debug,Clone, PartialEq)]
pub struct Chat{
    pub name: String,
    pub message: String,
}
use crate::THEME;
impl Chat{
    pub fn new(name : &str, messasge : &str) -> Self{
        return Self{
            name: name.to_string(),
            message: messasge.to_string()
        }
    }

    fn view(&self) -> Element<Message> {
        let is_ai = self.name != "User";
        let accent = match is_ai{
            true => THEME.palette().danger,
            false => THEME.palette().primary,
        };
        let name = container(text(&self.name).size(16)).style(container::Appearance{
            background: Some(Background::Color(accent)),
            border: Border::with_radius(5),
            text_color: Some(THEME.palette().background),
            ..Default::default()
        }).width(Length::Fill).padding(3);
        
        let replace_spaces_with_tabs = |text: &str|-> String {
          let re = Regex::new(r"(?m)^[ ]+").unwrap();
          re.replace_all(text, "\t\t").to_string()
        };
        
        let messagesplit = self.message.split_terminator("```");
        let mut messages = Vec::new();
        for (i, x) in messagesplit.enumerate(){
            if i % 2 != 0{
                let (l, c) = x.split_once("\n").unwrap();
                println!("{:?}", &c);
                let bg = THEME.palette().background;
                let c = replace_spaces_with_tabs(c);
                println!("{:?}", &c);

                let code = mouse_area(container(text(&c).size(18)).padding(8).style(container::Appearance{
                    background : Some(iced::Background::Color(darken_colour(bg, 0.02))),
                    border : Border::with_radius(5),
                    ..Default::default()
                }).width(Length::Fill)).on_press(Message::SaveToClipboard(c.to_string()));

                let lang = container(text(l).size(16)).padding(8).style(container::Appearance{
                    background : Some(iced::Background::Color(darken_colour(bg, 0.03))),
                    border : Border::with_radius(5),
                    ..Default::default()
                }).width(Length::Fill);


                let tip = container(text("Click to copy.").size(12)).padding(6).style(container::Appearance{
                    background : Some(iced::Background::Color(darken_colour(bg, 0.03))),
                    border : Border::with_radius(5),
                    ..Default::default()
                }).width(Length::Fill);

                let code_snippet = column![
                    lang,
                    code,
                    tip,
                ];

                messages.push(code_snippet.into());
                println!("y");
            }else{
                messages.push(text(x).size(18).into());
                println!("n");
            }
        }
        let mcontainer = container(column(messages)).padding(8);
        //.padding(8)
        let ret_message = container(column![name,mcontainer,].width(Length::Fill)).style(container::Appearance{
            border: Border{color: accent, width: 2.0, radius: 5.into()},
            ..Default::default()
        }).width(Length::FillPortion(5));

        let space = horizontal_space().width(Length::FillPortion(2));
        //let left_space = horizontal_space().width(Length::FillPortion(if is_ai {4} else {0}));
        //let right_space = horizontal_space().width(Length::FillPortion(if is_ai {0} else {4}));
        // let adjusted = match is_ai{
        //     true => row![space, ret_message],
        //     false => row![ret_message, space],
        // };

        let adjusted = ret_message;
        container(adjusted).into()
    }
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Chats ( pub Vec<Chat>, pub i32);

impl Chats{
    pub fn new() -> Self{

        Self(Vec::new(), Self::generate_id())
    }

    pub fn view(&self) -> Element<Message>{
        keyed_column(
            self.0
                .iter()
                .enumerate()
                .map(|(_, chat)| {
                    (
                        0,
                        chat.view()
                    )
                }),
        )
        .spacing(10)
        .into()
    }

    fn generate_id() -> i32{
        let mut rng = rand::thread_rng();
        let num = rng.gen_range(0..100000);
        return num;
    }

    pub fn new_with_chats(chats: Vec<Chat>) -> Self{
        return Self(chats, Self::generate_id());
    }

    pub fn get_preview(&self) -> String{
        return get_preview(self);
    }
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Save {
    pub ai_model : String,
    pub theme : String,
    pub chats : Vec<Chats>,
    pub last: i32,
}

impl Save {
    pub fn new(model : String, theme : String) -> Self{
        let chat = Chats::new();
        Self{
            ai_model: model,
            theme,
            chats: vec![chat.clone()],
            last: chat.1,
        }
    }

    pub fn view_chat(&self) -> Element<Message>{
        let index = self.get_index(self.last);
        if let Some(index) = index{
            return self.chats[index].view();
        }

        return text("Failed to get chat").into();
    }

    pub fn get_current_chat(&self) -> Option<Chats>{
        let index = self.get_index(self.last);
        if let Some(index) = index{
            return Some(self.chats[index].clone());
        }

        None
    }


    pub fn get_current_chat_num(&self) -> Option<usize>{
        let index = self.get_index(self.last);
        return index;
    }

    pub fn set_model(&mut self, model : String){
        self.ai_model = model;
    }


    pub fn set_theme(&mut self, theme : String){
        //THEME = theme.as_str();
    }

    pub fn set_chats(&mut self, chats : Vec<Chats>){
        self.chats = chats;
    }

    pub fn get_index(&self, id : i32) -> Option<usize>{
        for i in 0..self.chats.len(){
            if self.chats[i].1 == id{
                return Some(i);
            }
        }
        return None
    }

    pub fn ollama_from_chat(ollama : Arc<Mutex<Ollama>>, chat : Chats){
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
        tokio_runtime.block_on(Self::ollama_chat_async(ollama, chat));
    }

    pub async fn ollama_chat_async(ollama : Arc<Mutex<Ollama>>, chat : Chats){
        let mut o = ollama.lock().await;
        for c in chat.0{
            if c.name.as_str() == "AI"{
                o.add_assistant_response("default".to_string(), c.message.clone());
            }else{
                o.add_user_response("default".to_string(), c.message.clone());
            }
        }
    }

    pub fn update_chats(&mut self, chat : Chats){
        let mut new_chats = Vec::new();
        
        let mut found = false;
            // Iterate through existing chats
        for (i, existing_chat) in self.chats.iter().enumerate() {
            // Check for matching first message
            if existing_chat.1 == chat.clone().1 {
                // Update with new chat (converted to SaveChat)
                new_chats.push(chat.clone());
                println!("Adding");
                self.last = i as i32;
                found = true
            } else {
                // Existing chat doesn't match, keep existing
                new_chats.push(existing_chat.clone());
            }
        }

        if !found{
            new_chats.push(chat.clone());
        }


        if self.chats.len() <= new_chats.len(){
            // Update internal chats
            self.chats = new_chats;
        }
    }
    pub fn save(&self, path : &str){
        let writer = File::create(path);

        if let Ok(writer) = writer {
            let _ = serde_json::to_writer(writer, &self);
        }
    }

    pub fn replace(&mut self, save : Save){
        *self = save;
    }
    pub fn load(path : &str) -> Result<Self, String>{
        let mut reader = File::open(path);

        if let Ok(mut reader) = reader {
            let mut data = String::new();
            let _ = reader.read_to_string(&mut data).unwrap();

            let de_data = serde_json::from_str(&data);

            return match de_data {
                Ok(x) => Ok(x),
                Err(e) => Err(e.to_string()),
            };
        }

         return Err("Failed to open file".to_string());
    }

    pub fn get_current_preview(&self) -> String{
        match self.get_current_chat(){
            Some(x) => x.get_preview(),
            None => "New".to_string(),
        }
    }

    pub fn get_chat_previews(&self) -> Vec<String>{
        self.chats.clone().iter().map(|x| x.get_preview()).collect::<Vec<String>>()
    }
}
