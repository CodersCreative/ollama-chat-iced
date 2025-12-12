use iced::{Element, Task, widget::text_editor};
use ochat_types::{chats::Chat, settings::SettingsProvider};

use crate::{Application, Message};

#[derive(Debug, Clone)]
pub struct ChatsView {
    pub input: text_editor::Content,
    pub models: Vec<SettingsProvider>,
    pub messages: Vec<String>,
    pub chat: Chat,
}

#[derive(Debug, Clone)]
pub enum ChatsViewMessage {}

impl ChatsViewMessage {
    pub fn handle(self, app: &mut Application, id: u32) -> Task<Message> {
        match self {
            _ => Task::none(),
        }
    }
}

impl ChatsView {
    pub fn view<'a>(&'a self, app: &'a Application, id: u32) -> Element<'a, Message> {
        todo!()
    }
}
