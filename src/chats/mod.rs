pub mod chat;
pub mod message;
pub mod view;

use crate::common::Id;
use crate::utils::{get_path_settings, get_preview};
use chat::Chat;
use iced::widget::markdown;
use ollama_rs::generation::chat::ChatMessage;
use serde::{Deserialize, Serialize};
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
pub struct Relationship {
    index: usize,
    reason: Option<Reason>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Reason {
    Model(String),
    Regeneration,
    Sibling,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ChatTree {
    pub relationships: HashMap<usize, Vec<Relationship>>,
    pub chats: Vec<Chat>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SavedChat {
    pub chats: ChatTree,
    pub default_chats: Vec<usize>,
    pub default_tools: Vec<Id>,
    pub time: SystemTime,
}

impl Default for SavedChats {
    fn default() -> Self {
        Self(HashMap::from([(Id::new(), SavedChat::default())]))
    }
}

impl Default for SavedChat {
    fn default() -> Self {
        Self {
            chats: ChatTree::default(),
            default_chats: Vec::new(),
            default_tools: Vec::new(),
            time: SystemTime::now(),
        }
    }
}

impl SavedChat {
    pub fn to_mk(&self, chats: &[usize]) -> Vec<Vec<markdown::Item>> {
        let mut mk = Vec::new();

        for id in chats.iter() {
            if let Some(x) = &self.chats.chats.get(*id) {
                mk.push(Chat::generate_mk(x.content()))
            }
        }
        mk
    }

    pub fn new_with_chats_tools(chats: ChatTree, tools: Vec<Id>) -> Self {
        let mut saved = Self::new_with_chat_tree(chats);
        saved.default_tools = tools;
        saved
    }

    pub fn get_parent_index(&self, index: &usize) -> Option<usize> {
        self.chats
            .relationships
            .iter()
            .find(|x| x.1.iter().find(|x| &x.index == index).is_some())
            .map(|x| x.0.clone())
    }

    pub fn new_with_chats(chats: Vec<Chat>) -> Self {
        let mut saved = Self::default();
        saved.chats.chats = chats;
        saved
    }

    pub fn new_with_chat_tree(chats: ChatTree) -> Self {
        let mut saved = Self::default();
        saved.chats = chats;
        saved
    }

    pub fn get_preview(&self) -> (String, SystemTime) {
        return get_preview(self);
    }

    pub fn get_chats_with_reason(&self, chats: &[usize]) -> Vec<(usize, &Chat, Option<Reason>)> {
        let mut cts = Vec::new();

        for id in chats.iter() {
            let reason = if let Some(parent) = self.get_parent_index(id) {
                if let Some(relationship) = self.chats.relationships.get(&parent) {
                    relationship
                        .into_iter()
                        .find(|x| &x.index == id)
                        .map(|x| x.reason.clone())
                        .unwrap_or(None)
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(x) = self.chats.chats.get(*id) {
                cts.push((*id, x, reason))
            }
        }

        cts
    }

    pub fn get_path_from_index(&self, index: usize) -> Vec<usize> {
        let mut path = Vec::new();
        let mut parent = index;

        while let Some(p) = self.chats.relationships.get(&parent) {
            if let Some(first) = p.first() {
                parent = first.index;
                path.push(first.index);
            } else {
                return path;
            }
        }

        path
    }

    pub fn get_chat_messages_before(&self, chats: &[usize], before: usize) -> Vec<ChatMessage> {
        let mut cts = Vec::new();

        for id in chats[0..before].iter() {
            if let Some(x) = self.chats.chats.get(*id) {
                cts.push(Into::<ChatMessage>::into(x))
            }
        }
        cts
    }

    pub fn get_chat_messages(&self, chats: &[usize]) -> Vec<ChatMessage> {
        let mut cts = Vec::new();

        for id in chats.iter() {
            if let Some(x) = self.chats.chats.get(*id) {
                cts.push(Into::<ChatMessage>::into(x))
            }
        }
        cts
    }
}
