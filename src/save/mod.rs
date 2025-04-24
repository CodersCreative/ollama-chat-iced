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
    pub chats: HashMap<Id, SavedChats>,
}

impl Save {
    pub fn new() -> Self {
        Self {
            theme: None,
            use_panes: true,
            chats: HashMap::from([(Id::new(), SavedChats::new())]),
        }
    }

    pub fn view_chat<'a>(&'a self, chat: &'a Chats, id : &Id, app: &'a ChatApp) -> Element<'a, Message> {
        chat.view(app, id)
    }

    pub fn set_chats(&mut self, chats: HashMap<Id,SavedChats>) {
        self.chats = chats;
    }

    pub fn update_chats(&mut self, key : Id, chat: SavedChats) {
        self.chats.entry(key).and_modify(|x| *x = chat.clone()).or_insert(chat);
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

    pub fn get_chat_previews(&self) -> Vec<(Id, String, SystemTime)> {
        self.chats
            .clone()
            .iter()
            .map(|(id, x)| {
                let (title, time) = x.get_preview();
                (id.clone(), title, time)
            })
            .collect::<Vec<(Id, String, SystemTime)>>()
    }
}
