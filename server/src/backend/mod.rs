pub mod chats;
pub mod errors;
pub mod files;
pub mod generation;
pub mod options;
pub mod prompts;
pub mod providers;
pub mod settings;
pub mod user;
pub mod utils;

use crate::backend::{
    chats::{define_chats, previews::define_previews, relationships::define_message_relationships},
    errors::ServerError,
    files::define_files,
    messages::define_messages,
    options::{define_gen_options, relationships::define_gen_models},
    prompts::define_prompts,
    providers::{define_providers, ollama::models::define_ollama_models},
    settings::define_settings,
    user::{authenticate, define_users},
    utils::get_path_settings,
};
use axum::body::Body;
use chats::messages;
use std::sync::LazyLock;
use surrealdb::{
    Surreal,
    engine::local::{Db, RocksDb},
};

static CONN: LazyLock<Surreal<Db>> = LazyLock::new(Surreal::init);
const DATABASE: &str = "test";
const NAMESPACE: &str = "test";

pub async fn guard(
    req: axum::http::Request<Body>,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, ServerError> {
    let _ = authenticate(req.headers()).await?;
    Ok(next.run(req).await)
}

pub async fn connect_db() -> Result<(), ServerError> {
    let _ = CONN
        .connect::<RocksDb>(&get_path_settings("database".to_string()))
        .await?;
    Ok(())
}

pub async fn set_db() -> Result<(), ServerError> {
    let _ = CONN.use_ns(NAMESPACE).use_db(DATABASE).await?;
    Ok(())
}

pub async fn init_db() -> Result<(), ServerError> {
    let _ = connect_db().await?;
    let _ = set_db().await?;

    let _ = tokio::try_join![
        define_settings(),
        define_messages(),
        define_chats(),
        define_message_relationships(),
        define_ollama_models(),
        define_previews(),
        define_files(),
        define_prompts(),
        define_gen_options(),
        define_gen_models(),
        define_providers(),
        define_chats(),
        define_users(),
    ]?;

    Ok(())
}
