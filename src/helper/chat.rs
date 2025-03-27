use crate::{ chat::run_ollama, save::{chat::{Chat, Role},chats::Chats}, SAVE_FILE};
use crate::SideChats;

use iced::{widget::{markdown, Image}, Task};
use ollama_rs::generation::{chat::{ChatMessage, MessageRole}, images::{self, Image as OImage}};
use std::{path::PathBuf, sync::Arc};
use crate::{ChatApp, Message};

impl ChatApp{
    pub fn submit(&mut self, new : bool) -> Task<Message>{
        let ollama = Arc::clone(&self.logic.ollama);

        self.main_view.loading = true;
        if new{
            let chat = Chat{
                role: Role::User,
                message: self.main_view.input.clone(),
                images: self.main_view.images.clone(), 
            };
            self.markdown.push(Chat::generate_mk(&chat.message));
            let index = self.save.get_index(self.save.last).unwrap();
            self.save.chats[index].0.push(chat);
        }
        

        let chat = self.save.get_current_chat().unwrap();
        self.main_view.gen_chats = Arc::new(chat.get_chat_messages());
        let chat = Arc::clone(&self.main_view.gen_chats);
        
        Task::perform(run_ollama(chat ,ollama, self.get_model()), Message::Received)
    }

    pub fn received(&mut self, result : ChatMessage) -> Task<Message>{
        self.main_view.loading = false;
        let index = self.save.get_index(self.save.last);
        let images = match result.images{
            Some(x) => x.iter().map(|x| {
                PathBuf::from("")
            }).collect(),
            None => Vec::new(),
        };

        let chat = Chat{
            role: Role::AI,
            message: result.content.trim().to_string(),
            images, 
        };

        match index{
            Some(x) => {
                self.markdown.push(Chat::generate_mk(&chat.message));
                self.save.chats[x].0.push(chat);
            },
            None => {
                self.save.chats.push(Chats::new_with_chats(vec![chat]))
            },
        }

        self.main_view.input = String::new();
        self.main_view.images = Vec::new();
        self.save.save(SAVE_FILE);
        self.main_view.chats = SideChats::new(self.save.get_chat_previews());

        Task::none()
    }
}
