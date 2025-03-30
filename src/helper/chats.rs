use crate::{
    chat::get_model, save::chats::{Chats, ChatsMessage, SavedChats}, sidebar::chats::Chats as SideChats, SAVE_FILE
};

use iced::Task;
use tokio::sync::Mutex as TMutex;
use std::sync::Arc;
use crate::{ChatApp, Message};

impl ChatsMessage{
    pub fn change_chat(&self, o_index : usize, chats : Chats, app : &mut ChatApp,) -> Task<Message>{
        //self.save.last = ;
        let index = Chats::get_index(app, chats.id);
        app.main_view.chats[index].saved_id = app.save.chats[o_index].1;
        app.logic.chat = Some(index);
        //let chat = self.save.get_current_chat();
        
        //let s_index = chats.get_saved_index(app).unwrap();
        //if let Some(chat) = chat{
        app.main_view.chats[index].markdown = app.save.chats[o_index].to_mk();

        //}
        app.save.save(SAVE_FILE);

        Task::none()
    }



    pub fn new_chat(app : &mut ChatApp, id : i32) -> Task<Message>{
        let chat = SavedChats::new();
        app.save.chats.push(chat.clone());
        //self.save.last = chat.1.clone();
        //self.markdown = Vec::new();
        //self.logic.chat = Some(self.save.chats.len() - 1);
        app.main_view.side_chats = SideChats::new(app.save.get_chat_previews());
        Task::none()
    }
}

impl ChatApp{

    pub fn remove_chat(&mut self, o_index : usize) -> Task<Message>{
        for c in &self.main_view.chats{
            if c.loading{
                return Task::none();
            }
        }
        let id = self.save.chats[o_index].1;
        self.save.chats.remove(o_index);
        //self.save.last = ;
        for c in &mut self.main_view.chats{
            if c.saved_id == id{
                c.saved_id = self.save.chats.last().unwrap().1.clone();
                c.markdown = Vec::new();
            }
        } 
        //self.logic.ollama = Arc::new(TMutex::new(get_model()));
        //self.markdown = Vec::new();
        self.logic.chat = Some(self.save.chats.len() - 1);
        self.main_view.side_chats = SideChats::new(self.save.get_chat_previews());
        Task::none()
    }
}
