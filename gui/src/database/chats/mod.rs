use crate::{chats::chat::Chat, database::CONN};

pub async fn get_chat_message(id: String) -> Chat {
    CONN.select(("messages", id))
}

pub async fn delete_chat_message(id: String) {
    CONN.delete(("messages", id))
}
