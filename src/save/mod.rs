pub mod chat;
pub mod chats;

use crate::chats::Chats;
use crate::common::Id;
use crate::utils::get_path_settings;
use crate::{ChatApp, Message};
use chats::SavedChats;
use iced::Element;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::time::SystemTime;
use std::{fs::File, io::Read};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Save {
    pub theme: Option<usize>,
    pub use_panes: bool,
    pub chats: Vec<SavedChats>,
}

impl Save {
    pub fn new() -> Self {
        let chat = SavedChats::new();
        Self {
            theme: None,
            use_panes: true,
            chats: vec![chat.clone()],
        }
    }

    pub fn view_chat<'a>(&'a self, chat: &'a Chats, app: &'a ChatApp) -> Element<'a, Message> {
        chat.view(app)
    }

    // pub fn get_current_chat(&self) -> Option<SavedChats>{
    //     let index = self.get_index(self.last);
    //     if let Some(index) = index{
    //         return Some(self.chats[index].clone());
    //     }
    //
    //     None
    // }
    //
    //
    // pub fn get_current_chat_num(&self) -> Option<usize>{
    //     let index = self.get_index(self.last);
    //     return index;
    // }

    pub fn set_chats(&mut self, chats: Vec<SavedChats>) {
        self.chats = chats;
    }

    pub fn get_index(&self, id: Id) -> Option<usize> {
        for i in 0..self.chats.len() {
            if self.chats[i].1 == id {
                return Some(i);
            }
        }
        return None;
    }

    pub fn update_chats(&mut self, chat: SavedChats) {
        let mut new_chats = Vec::new();

        let mut found = false;
        for (i, existing_chat) in self.chats.iter().enumerate() {
            if existing_chat.1 == chat.clone().1 {
                new_chats.push(chat.clone());
                println!("Adding");
                // self.last = i as i32;
                found = true
            } else {
                new_chats.push(existing_chat.clone());
            }
        }

        if !found {
            new_chats.push(chat.clone());
        }

        if self.chats.len() <= new_chats.len() {
            self.chats = new_chats;
        }
    }

    pub fn save(&self, path: &str) {
        let path = get_path_settings(path.to_string());
        let writer = File::create(path);

        if let Ok(writer) = writer {
            let _ = serde_json::to_writer_pretty(writer, &self);
        }
    }

    pub fn replace(&mut self, save: Save) {
        *self = save;
    }

    pub fn load(path: &str) -> Result<Self, String> {
        let path = get_path_settings(path.to_string());
        let reader = File::open(path);

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

    // pub fn get_current_preview(&self) -> (String, SystemTime){
    //     match self.get_current_chat(){
    //         Some(x) => x.get_preview(),
    //         None => ("New".to_string(), SystemTime::now()),
    //     }
    // }

    pub fn get_chat_previews(&self) -> Vec<(String, SystemTime)> {
        self.chats
            .clone()
            .iter()
            .map(|x| x.get_preview())
            .collect::<Vec<(String, SystemTime)>>()
    }
}
