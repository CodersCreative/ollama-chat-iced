use crate::{chats::State, common::Id, save::chats::{ChatsMessage, SavedChats}, sidebar::chats::Chats as SideChats, SAVE_FILE};
use iced::Task;
use crate::{ChatApp, Message};

impl ChatsMessage{
    pub fn new_chat(app : &mut ChatApp, _id : Id) -> Task<Message>{
        let chat = SavedChats::new();
        app.save.chats.push(chat.clone());
        app.regenerate_side_chats();
        Task::none()
    }
}

impl ChatApp{
    pub fn remove_chat(&mut self, o_index : usize) -> Task<Message>{
        for c in self.main_view.get_chats(){
            if c.get_state() != &State::Idle{
                return Task::none();
            }
        }
        
        let id = self.save.chats[o_index].1;
        self.save.chats.remove(o_index);
        
        for c in self.main_view.get_chats_mut(){
            if c.get_saved_chat() == &id{
                c.set_saved_chat(self.save.chats.last().unwrap().1.clone());
                c.set_markdown(Vec::new());
            }
        } 
        
        // self.logic.chat = Some(self.save.chats.len() - 1);
        self.regenerate_side_chats();
        Task::none()
    }
    

    pub fn regenerate_side_chats(&mut self) {
        self.main_view.set_side_chats(SideChats::new(self.save.get_chat_previews()));
    }
}
