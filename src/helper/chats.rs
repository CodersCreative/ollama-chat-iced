use crate::{
    chat::{get_model}, save::chats::Chats, sidebar::chats::Chats as SideChats, SAVE_FILE
};

use iced::Task;
use tokio::sync::Mutex as TMutex;
use std::sync::Arc;
use crate::{ChatApp, Message};

impl ChatApp{
    pub fn change_chat(&mut self, index : usize) -> Task<Message>{
        self.save.last = self.save.chats[index].1;
        self.logic.chat = Some(index);
        let chat = self.save.get_current_chat();
        if let Some(chat) = chat{
            self.markdown = chat.to_mk();

        }
        self.save.save(SAVE_FILE);

        Task::none()
    }

    pub fn remove_chat(&mut self, index : usize) -> Task<Message>{
        self.save.chats.remove(index);
        self.save.last = self.save.chats.last().unwrap().1.clone();
        self.logic.ollama = Arc::new(TMutex::new(get_model()));
        self.markdown = Vec::new();
        self.logic.chat = Some(self.save.chats.len() - 1);
        self.main_view.chats = SideChats::new(self.save.get_chat_previews());
        Task::none()
    }

    pub fn new_chat(&mut self) -> Task<Message>{
        let chat = Chats::new();
        self.save.chats.push(chat.clone());
        self.save.last = chat.1.clone();
        self.logic.ollama = Arc::new(TMutex::new(get_model()));
        self.markdown = Vec::new();
        self.logic.chat = Some(self.save.chats.len() - 1);
        self.main_view.chats = SideChats::new(self.save.get_chat_previews());
        Task::none()
    }
}
