use crate::chats::{self, messages, previews, relationships};
use axum::{
    Router,
    routing::{get, post, put},
};

pub fn routes() -> Router {
    Router::new()
        .route("/message/", post(messages::create_message))
        .route(
            "/message/parent/{parent}",
            post(messages::create_message_with_parent).get(messages::get_message_list_from_parent),
        )
        .route(
            "/message/parent/{parent}/default/",
            get(messages::get_default_message_list_from_parent),
        )
        .route(
            "/message/parent/{parent}/change/",
            get(messages::get_can_change_list_from_parent),
        )
        .route(
            "/message/parent/{parent}/change/default/",
            get(messages::get_default_can_change_list_from_parent),
        )
        .route(
            "/message/parent/{parent}/all/",
            get(messages::list_all_messages_from_parent),
        )
        .route(
            "/message/{id}",
            get(messages::read_message)
                .put(messages::update_message)
                .delete(messages::delete_message),
        )
        .route("/message/{id}/change/", get(messages::get_can_change))
        .route("/preview/all/", get(previews::list_all_previews))
        .route("/preview/search/{search}", get(previews::search_previews))
        .route(
            "/preview/{id}",
            get(previews::get_preview).put(previews::update_preview),
        )
        .route("/chat/", post(chats::create_chat))
        .route("/chat/{id}/root/{root}", put(chats::set_chat_root))
        .route("/chat/all/", get(chats::list_all_chats))
        .route(
            "/chat/{id}",
            get(chats::get_chat)
                .put(chats::update_chat)
                .delete(chats::delete_chat),
        )
        .route(
            "/relationship/parent/{parent}/all/",
            get(relationships::list_all_message_relationships_from_parent),
        )
        .route(
            "/relationship/",
            post(relationships::create_message_relationship),
        )
        .route(
            "/relationship/all/",
            get(relationships::list_all_message_relationships),
        )
        .route(
            "/relationship/{id}",
            get(relationships::get_message_relationship)
                .put(relationships::update_message_relationship)
                .delete(relationships::delete_message_relationship),
        )
}
