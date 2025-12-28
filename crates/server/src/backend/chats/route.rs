use crate::backend::chats::{self, messages, previews};
use axum::{
    Router,
    routing::{get, post, put},
};

pub fn routes() -> Router {
    Router::new()
        .route("/message/", post(messages::create_message))
        .route(
            "/message/parent/{parent}",
            post(messages::create_message_with_parent).get(messages::list_all_messages_from_parent),
        )
        .route(
            "/message/parent/{parent}/default/",
            get(messages::get_default_message_list_from_parent),
        )
        .route(
            "/message/parent/{id}/all/",
            get(messages::get_all_messages_from_root),
        )
        .route(
            "/message/{id}",
            get(messages::read_message)
                .put(messages::update_message)
                .delete(messages::delete_message),
        )
        .route("/message/all/", get(messages::list_all_messages))
        .route("/preview/all/", get(previews::list_all_previews))
        .route("/preview/search/{search}", get(previews::search_previews))
        .route(
            "/preview/{id}",
            get(previews::get_preview).put(previews::update_preview),
        )
        .route("/chat/", post(chats::create_chat))
        .route("/chat/branch/", post(chats::branch_new_chat))
        .route("/chat/{id}/root/{root}", put(chats::set_chat_root))
        .route("/chat/all/", get(chats::list_all_chats))
        .route(
            "/chat/{id}",
            get(chats::get_chat)
                .put(chats::update_chat)
                .delete(chats::delete_chat),
        )
}
