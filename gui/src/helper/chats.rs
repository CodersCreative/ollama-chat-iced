use std::time::SystemTime;

use crate::chats::chat::Role;
use crate::chats::message::ChatsMessage;
use crate::chats::view::State;
use crate::chats::{SavedChat, CHATS_FILE};
use crate::common::Id;
use crate::previews::{generate_preview, SavedPreview};
use crate::{ChatApp, Message};
use iced::Task;

impl ChatsMessage {
    pub fn new_chat(app: &mut ChatApp, id: Id) -> Task<Message> {
        let saved = Id::new();
        app.chats.0.insert(saved.clone(), SavedChat::default());
        Self::changed_saved(app, id, saved);
        app.regenerate_side_chats(vec![id])
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
        self.previews.0.remove(&key);
        self.main_view
            .set_side_chats(self.previews.get_side_chats());
        Task::none()
    }

    pub fn regenerate_side_chats(&mut self, ids: Vec<Id>) -> Task<Message> {
        let provider = self.logic.get_random_provider().unwrap();
        let model = self.logic.models[0].clone();

        let _ = self.previews.0.retain(|k, _| self.chats.0.contains_key(k));

        let data: Vec<(Id, Vec<(String, Role)>, &SystemTime)> = if ids.is_empty() {
            self.chats
                .0
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.1.get_chat_message_texts(&x.1.default_chats),
                        &x.1.time,
                    )
                })
                .collect()
        } else {
            self.chats
                .0
                .iter()
                .filter(|x| ids.contains(x.0))
                .map(|x| {
                    (
                        x.0.clone(),
                        x.1.get_chat_message_texts(&x.1.default_chats),
                        &x.1.time,
                    )
                })
                .collect()
        };

        for chat in data.iter() {
            if !self.previews.0.contains_key(&chat.0) {
                let _ = self.previews.0.insert(
                    chat.0,
                    SavedPreview {
                        text: String::from("New"),
                        time: chat.2.clone(),
                    },
                );
            }
        }
        self.main_view
            .set_side_chats(self.previews.get_side_chats());

        let mut tasks = Vec::new();

        for chat in data.into_iter() {
            let provider = provider.clone();
            let model = model.clone();
            let content = chat.0.clone();
            tasks.push(Task::future(async move {
                Message::SetPreviews(generate_preview(chat.1, content, model, provider).await)
            }));
        }

        Task::batch(tasks)
    }
}
