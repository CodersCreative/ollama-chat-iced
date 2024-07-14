pub mod chat;
pub mod chats;

use iced::{theme::Palette, widget::text};
use iced::Element;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{fs::File, io::Read};
use ollama_rs::Ollama;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::Message;
use chats::Chats;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Save {
    pub ai_model : String,
    pub theme : Option<usize>,
    pub chats : Vec<Chats>,
    pub last: i32,
    pub code_indent : usize, 
}

impl Save {
    pub fn new(model : String) -> Self{
        let chat = Chats::new();
        Self{
            ai_model: model,
            theme : None,
            chats: vec![chat.clone()],
            last: chat.1,
            code_indent: 8,
        }
    }

    pub fn view_chat(&self, palette : Palette) -> Element<Message>{
        let index = self.get_index(self.last);
        if let Some(index) = index{
            return self.chats[index].view(palette, self.code_indent);
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
