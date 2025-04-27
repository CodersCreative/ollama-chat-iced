use crate::{
    chats::State,
    common::Id,
    save::chats::{ChatsMessage, SavedChats},
    sidebar::chats::Chats as SideChats,
};
use crate::{ChatApp, Message};
use iced::Task;

impl ChatsMessage {
    pub fn new_chat(app: &mut ChatApp, _id: Id) -> Task<Message> {
        let chat = SavedChats::new();
        app.save.chats.insert(Id::new(), chat.clone());
        app.regenerate_side_chats();
        Task::none()
    }
}

impl ChatApp {
    pub fn remove_chat(&mut self, key: Id) -> Task<Message> {
        self.save.chats.remove(&key);

        if let Some(saved) = self.save.chats.iter().last() {
            for c in self.main_view.chats_mut() {
                if c.1.saved_chat() == &key {
                    c.1.set_saved_chat(saved.0.clone());
                    c.1.set_markdown(Vec::new());
                }
            }
        }

        self.regenerate_side_chats();
        Task::none()
    }

    pub fn regenerate_side_chats(&mut self) {
        self.main_view
            .set_side_chats(SideChats::new(self.save.get_chat_previews()));
    }
}
