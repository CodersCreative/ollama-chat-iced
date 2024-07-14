use crate::{
    save::{Save, chats::Chats},
    sidebar::chats::Chats as SideChats,
    chat::get_model,
    SAVE_FILE
};

use iced::Command;
use tokio::sync::Mutex as TMutex;
use std::sync::Arc;
use crate::{ChatApp, Message};

impl ChatApp{
    pub fn change_chat(&mut self, index : usize) -> Command<Message>{
        self.save.last = self.save.chats[index].1;
        self.logic.chat = Some(index);
        let chat = self.save.get_current_chat();
        if let Some(chat) = chat{
            let ollama = Arc::clone(&self.logic.ollama);
            Save::ollama_from_chat(ollama, chat);
        }
        self.save.save(SAVE_FILE);
        Command::none()
    }

    pub fn remove_chat(&mut self, index : usize) -> Command<Message>{
        self.save.chats.remove(index);
        self.save.last = self.save.chats.last().unwrap().1.clone();
        self.logic.ollama = Arc::new(TMutex::new(get_model()));
        self.logic.chat = Some(self.save.chats.len() - 1);
        self.main_view.chats = SideChats::new(self.save.get_chat_previews());
        Command::none()
    }

    pub fn new_chat(&mut self) -> Command<Message>{
        let chat = Chats::new();
        self.save.chats.push(chat.clone());
        self.save.last = chat.1.clone();
        self.logic.ollama = Arc::new(TMutex::new(get_model()));
        self.logic.chat = Some(self.save.chats.len() - 1);
        self.main_view.chats = SideChats::new(self.save.get_chat_previews());
        Command::none()
    }
}
