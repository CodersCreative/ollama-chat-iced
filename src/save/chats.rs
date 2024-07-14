use iced::{theme::Palette, widget::keyed_column};
use iced::Element;
use serde::{Deserialize, Serialize};
use crate::{utils::get_preview, Message};
use rand::Rng;

use super::chat::Chat;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Chats ( pub Vec<Chat>, pub i32);

impl Chats{
    pub fn new() -> Self{
        Self(Vec::new(), Self::generate_id())
    }

    pub fn view(&self, palette : Palette, indent : usize) -> Element<Message>{
        keyed_column(
            self.0
                .iter()
                .enumerate()
                .map(|(_, chat)| {
                    (
                        0,
                        chat.view(palette, indent)
                    )
                }),
        )
        .spacing(10)
        .into()
    }

    fn generate_id() -> i32{
        let mut rng = rand::thread_rng();
        let num = rng.gen_range(0..100000);
        return num;
    }

    pub fn new_with_chats(chats: Vec<Chat>) -> Self{
        return Self(chats, Self::generate_id());
    }

    pub fn get_preview(&self) -> String{
        return get_preview(self);
    }
}
