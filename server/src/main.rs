pub mod chats;
pub mod errors;
pub mod generation;
pub mod providers;
pub mod utils;
use std::sync::LazyLock;

use surrealdb::{
    Surreal,
    engine::local::{Db, RocksDb},
};

static CONN: LazyLock<Surreal<Db>> = LazyLock::new(Surreal::init);

use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    chats::define_chat, errors::ServerError, providers::define_providers, utils::get_path_settings,
};

#[tokio::main]
async fn main() {
    init_db().await.unwrap();
    let app = Router::new()
        .route("/message/", post(chats::create_chat_message))
        .route("/message/all/", get(chats::list_all_chat_messages))
        .route(
            "/message/{id}",
            get(chats::read_chat_message)
                .put(chats::update_chat_message)
                .delete(chats::delete_chat_message),
        )
        .route("/provider/", post(providers::add_provider))
        .route("/provider/all/", get(providers::list_all_providers))
        .route(
            "/provider/{id}",
            get(providers::read_provider)
                .put(providers::update_provider)
                .delete(providers::delete_provider),
        )
        .route("/generation/text/run/", get(generation::text::run))
        .route(
            "/generation/text/stream/",
            get(generation::text::stream::run),
        );
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

    let _ = define_chat().await?;
    define_providers().await
}
