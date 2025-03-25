use crate::{
    save::{chat::Chat, chats::Chats},
    sidebar::chats::Chats as SideChats,
    chat::run_ollama,
    SAVE_FILE
};

use iced::{widget::markdown, Task};
use std::sync::Arc;
use crate::{ChatApp, Message};

impl ChatApp{
    pub fn submit(&mut self) -> Task<Message>{
        let ollama = Arc::clone(&self.logic.ollama);
        let input = self.main_view.input.clone();
        self.main_view.loading = true;
        Task::perform(run_ollama(input, ollama, self.get_model()), Message::Received)
    }

    pub fn received(&mut self, result : String) -> Task<Message>{
        self.main_view.loading = false;
        let index = self.save.get_index(self.save.last);

        let chats = vec![Chat{
            name : "User".to_owned(),
            message : self.main_view.input.clone(),
        },
        Chat{
            name : "AI".to_owned(),
            message : result.trim().to_string(),
        }];

        match index{
            Some(x) => {
                self.save.chats[x].0.extend(chats.clone());
                let mut chats : Vec<Vec<markdown::Item>> = chats.iter().map(|x| Chat::generate_mk(&x.message)).collect();
                self.markdown.append(&mut chats);
            },
            None => {
                self.save.chats.push(Chats::new_with_chats(chats))
            },
        }
        self.main_view.input = String::new();
        self.save.save(SAVE_FILE);
        self.main_view.chats = SideChats::new(self.save.get_chat_previews());

        Task::none()
    }
}
