pub mod chat;
pub mod message;
pub mod view;
pub mod tree;

use crate::utils::{get_path_settings, get_preview};
use crate::{common::Id, llm::Tools};
use chat::{Chat, ChatBuilder, Role};
use iced::widget::markdown;
use ollama_rs::generation::chat::ChatMessage;
use serde::{Deserialize, Serialize};
use tree::ChatTree;
use std::fs::File;
use std::io::Read;
use std::{collections::HashMap, time::SystemTime};

pub const CHATS_FILE: &str = "chat.json";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SavedChats(pub HashMap<Id, SavedChat>);

impl SavedChats {
    pub fn set_chats(&mut self, chats: HashMap<Id, SavedChat>) {
        self.0 = chats;
    }

    pub fn update_chats(&mut self, key: Id, chat: SavedChat) {
        self.0
            .entry(key)
            .and_modify(|x| *x = chat.clone())
            .or_insert(chat);
    }

    pub fn get_chat_previews(&self) -> Vec<(Id, String, SystemTime)> {
        self.0
            .clone()
            .iter()
            .map(|(id, x)| {
                let (title, time) = x.get_preview();
                (id.clone(), title, time)
            })
            .collect::<Vec<(Id, String, SystemTime)>>()
    }

    pub fn save(&self, path: &str) {
        let path = get_path_settings(path.to_string());
        let writer = File::create(path);

        if let Ok(writer) = writer {
            let _ = serde_json::to_writer_pretty(writer, &self);
        }
    }

    pub fn load(path: &str) -> Result<Self, String> {
        let path = get_path_settings(path.to_string());
        let reader = File::open(path);

        if let Ok(mut reader) = reader {
            let mut data = String::new();
            let _ = reader
                .read_to_string(&mut data)
                .map_err(|e| e.to_string())?;

            let de_data = serde_json::from_str(&data);

            return match de_data {
                Ok(x) => Ok(x),
                Err(e) => Err(e.to_string()),
            };
        }

        return Err("Failed to open file".to_string());
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SavedChat{
    pub chats : ChatTree, 
    pub tools : Vec<Tools>, 
    pub time : SystemTime,
    // pub latest_node: Option<NodeId>,
}

#[derive(Default, Debug)]
pub struct TooledOptions {
    pub chats: Vec<ChatMessage>,
    pub tools: Vec<Tools>,
}

impl Default for SavedChats {
    fn default() -> Self {
        Self(HashMap::from([(Id::new(), SavedChat::default())]))
    }
}

impl Default for SavedChat{
    fn default() -> Self {
        Self{
            chats: ChatTree::new(ChatBuilder::default().content(String::from("Chat Started!")).role(Role::System).build().unwrap()),
            tools : Vec::new(),
            time : SystemTime::now(),
        }
    }
}

impl SavedChat {
    pub fn to_mk(&self) -> Vec<Vec<markdown::Item>> {
        return self.chats.get_full_history() 
            .iter()
            .map(|x| Chat::generate_mk(&x.content()))
            .collect();
    }

    pub fn new_with_chats_tools(chats: ChatTree, tools: Vec<Tools>) -> Self {
        let mut saved = Self::new_with_chats(chats);
        saved.tools = tools;
        saved
    }

    pub fn new_with_chats(chats: ChatTree) -> Self {
        let mut saved = Self::default();
        saved.chats = chats;
        saved
    }

    pub fn get_preview(&self) -> (String, SystemTime) {
        return get_preview(self);
    }

    pub fn get_chat_messages(&self) -> Vec<ChatMessage> {
        self.chats.get_full_history().iter().map(|x| (*x).into()).collect()
    }
    
}
