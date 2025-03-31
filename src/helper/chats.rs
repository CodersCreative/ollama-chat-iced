use crate::{
    save::chats::{Chats, ChatsMessage, SavedChats}, sidebar::chats::Chats as SideChats, SAVE_FILE
};

use iced::Task;
use crate::{ChatApp, Message};

impl ChatsMessage{
    //pub fn change_chat(&self, o_index : usize, id : i32, app : &mut ChatApp,) -> Task<Message>{
    //    let mut chats = Chats::get_from_id_mut(app, id);
    //
    //    if !chats.loading{
    //        chats.saved_id = app.save.chats[o_index].1;
    //        chats.markdown = app.save.chats[o_index].to_mk();
    //        app.logic.chat = Some(Chats::get_index(app, id));
    //        app.save.save(SAVE_FILE);
    //    }
    //
    //    Task::none()
    //}



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
            if c.loading{
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
