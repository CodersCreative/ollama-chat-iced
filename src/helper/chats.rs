use crate::chats::message::ChatsMessage;
use crate::chats::view::State;
use crate::chats::{SavedChat, CHATS_FILE};
use crate::{common::Id, sidebar::chats::SideChats};
use crate::{ChatApp, Message};
use iced::Task;

impl ChatsMessage {
    pub fn new_chat(app: &mut ChatApp, id: Id) -> Task<Message> {
        let saved = Id::new();
        app.chats.0.insert(saved.clone(), SavedChat::default());
        app.regenerate_side_chats();
        Self::changed_saved(app, id, saved);
        Task::none()
    }

    pub fn changed_saved(app: &mut ChatApp, id: Id, saved: Id) {
        app.chats.save(CHATS_FILE);
        app.main_view.update_chat(&id, |chat| {
            if let Some(chat) = chat {
                chat.set_markdown(app.chats.0.get(&saved).unwrap().to_mk(&chat.chats()));
                chat.set_saved_chat(saved);
                chat.set_state(State::Idle);
                *chat.chats_mut() = app.chats.0.get(&saved).unwrap().default_chats.clone();
                chat.set_edit_index(None);
            }
        });
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

        self.chats.save(CHATS_FILE);
        self.regenerate_side_chats();
        Task::none()
    }

    pub fn regenerate_side_chats(&mut self) {
        self.main_view
            .set_side_chats(SideChats::new(self.chats.get_chat_previews()));
    }
}
