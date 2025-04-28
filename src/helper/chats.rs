use crate::chats::message::ChatsMessage;
use crate::chats::SavedChat;
use crate::{common::Id, sidebar::chats::SideChats};
use crate::{ChatApp, Message};
use iced::Task;

impl ChatsMessage {
    pub fn new_chat(app: &mut ChatApp, _id: Id) -> Task<Message> {
        let chat = SavedChat::new();
        app.chats.0.insert(Id::new(), chat.clone());
        app.regenerate_side_chats();
        Task::none()
    }
}

impl ChatApp {
    pub fn remove_chat(&mut self, key: Id) -> Task<Message> {
        self.chats.0.remove(&key);

        if let Some(saved) = self.chats.0.iter().last() {
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
            .set_side_chats(SideChats::new(self.chats.get_chat_previews()));
    }
}
