pub mod chats;
pub mod errors;
pub mod utils;
use std::sync::LazyLock;

use surrealdb::{
    Surreal,
    engine::local::{Db, RocksDb},
};

static CONN: LazyLock<Surreal<Db>> = LazyLock::new(Surreal::init);

use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::{chats::define_chat, errors::ServerError, utils::get_path_settings};

#[tokio::main]
async fn main() {
    init_db().await.unwrap();
    let app = Router::new()
        .route("/message/", post(chats::create_chat_message))
        .route("/message/{id}", get(chats::read_chat_message))
        .route("/message/{id}", put(chats::update_chat_message))
        .route("/message/{id}", delete(chats::delete_chat_message))
        .route("/message/all/", get(chats::list_all_chat_messages));

    let listener = tokio::net::TcpListener::bind("localhost:1212")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

pub async fn init_db() -> Result<(), ServerError> {
    let _ = CONN
        .connect::<RocksDb>(&get_path_settings("database".to_string()))
        .await?;

    let _ = CONN.use_ns("test").use_db("test").await?;

    define_chat().await
}
