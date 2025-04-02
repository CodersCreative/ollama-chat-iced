use crate::{
    save::chats::{Chats, ChatsMessage, SavedChats, State}, sidebar::chats::Chats as SideChats, SAVE_FILE
};

use iced::Task;
use crate::{ChatApp, Message};

impl ChatsMessage{
    pub fn new_chat(app : &mut ChatApp, id : i32) -> Task<Message>{
        let chat = SavedChats::new();
        app.save.chats.push(chat.clone());
        app.main_view.side_chats = SideChats::new(app.save.get_chat_previews());
        Task::none()
    }
}

impl ChatApp{

    pub fn remove_chat(&mut self, o_index : usize) -> Task<Message>{
        for c in &self.main_view.chats{
            if c.state != State::Idle{
                return Task::none();
            }
        }
        let id = self.save.chats[o_index].1;
        self.save.chats.remove(o_index);
        for c in &mut self.main_view.chats{
            if c.saved_id == id{
                c.saved_id = self.save.chats.last().unwrap().1.clone();
                c.markdown = Vec::new();
            }
        } 
        self.logic.chat = Some(self.save.chats.len() - 1);
        self.main_view.side_chats = SideChats::new(self.save.get_chat_previews());
        Task::none()
    }
}
