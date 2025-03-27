use std::time::SystemTime;

use iced::widget::markdown;
use iced::widget::keyed_column;
use iced::Element;
use serde::{Deserialize, Serialize};
use crate::ChatApp;
use crate::{utils::get_preview, Message};
use rand::Rng;
use std::time::Instant;
use std::error::Error;
use tokio::sync::Mutex;
use std::sync::Arc;

use ollama_rs::{
    generation::chat::{
        request::ChatMessageRequest, ChatMessage,
    }, Ollama
};
use super::chat::Chat;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Chats ( pub Vec<Chat>, pub i32, pub SystemTime);

impl Chats{
    pub fn new() -> Self{
        Self(Vec::new(), Self::generate_id(), SystemTime::now())
    }

    pub fn view<'a>(&'a self, app : &'a ChatApp) -> Element<'a, Message>{
        keyed_column(
            self.0
                .iter()
                .enumerate()
                .map(|(i, chat)| {
                    (
                        0,
                        chat.view(&app.markdown[i], &app.theme())
                    )
                }),
        )
        .spacing(10)
        .into()
    }

    pub fn to_mk(&self) -> Vec<Vec<markdown::Item>>{
        return self.0.iter().map(|x| Chat::generate_mk(&x.message)).collect();
    }

    fn generate_id() -> i32{
        let mut rng = rand::thread_rng();
        let num = rng.gen_range(0..100000);
        return num;
    }

    pub fn new_with_chats(chats: Vec<Chat>) -> Self{
        return Self(chats, Self::generate_id(), SystemTime::now());
    }

    pub fn get_preview(&self) -> (String, SystemTime){
        return get_preview(self);
    }

    pub fn get_chat_messages(&self) -> Vec<ChatMessage>{
        self.0.iter().map(|x| {
            x.into()
        }).collect()
    }
}
