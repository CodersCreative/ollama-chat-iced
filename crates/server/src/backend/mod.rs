pub mod chats;
pub mod errors;
pub mod files;
pub mod folders;
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
    folders::define_folders,
    messages::define_messages,
    options::{define_gen_options, relationships::define_gen_models},
    prompts::define_prompts,
    providers::{
        add_default_providers, define_providers,
        ollama::models::{add_all_ollama_models, define_ollama_models},
    },
    settings::define_settings,
    user::{authenticate, define_users},
    utils::get_path_settings,
};
use axum::{Router, body::Body, middleware};
use chats::messages;
use clap::Parser;
use ochat_types::WORD_ART;
use std::sync::LazyLock;
use surrealdb::{
    Surreal,
    engine::local::{Db, RocksDb},
};

static CONN: LazyLock<Surreal<Db>> = LazyLock::new(Surreal::init);
const DATABASE: &str = "test";
const NAMESPACE: &str = "test";

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    #[arg(short, long)]
    url: Option<String>,
}

pub async fn start_server<F: FnOnce(String) -> Router>(router_fn: F) {
    let args = Arguments::parse();
    init_db().await.unwrap();
    let api = Router::new().merge(user::route::auth_routes());

    let api_protected = Router::new()
        .merge(user::route::routes())
        .merge(chats::route::routes())
        .merge(files::route::routes())
        .merge(generation::route::routes())
        .merge(options::route::routes())
        .merge(prompts::route::routes())
        .merge(providers::route::routes())
        .merge(settings::route::routes())
        .merge(folders::route::routes())
        .route_layer(middleware::from_fn(guard));

    let mut url = args.url.unwrap_or("localhost:1212".to_string());

    if url.is_empty() {
        url = "localhost:1212".to_string();
    }

    url = url.replace("localhost", "127.0.0.1");
    url = url.trim_end_matches("/api").to_string();

    let app = router_fn(url.clone()).nest("/api", Router::new().merge(api).merge(api_protected));

    println!("{}", WORD_ART);
    println!("Starting server at '{}'.", url);

    let listener = tokio::net::TcpListener::bind(url).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

pub async fn guard(
    req: axum::http::Request<Body>,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, ServerError> {
    authenticate(req.headers()).await?;
    Ok(next.run(req).await)
}

pub async fn connect_db() -> Result<(), ServerError> {
    CONN.connect::<RocksDb>(&get_path_settings("database".to_string()))
        .await?;
    Ok(())
}

pub async fn set_db() -> Result<(), ServerError> {
    CONN.use_ns(NAMESPACE).use_db(DATABASE).await?;
    Ok(())
}

pub async fn init_db() -> Result<(), ServerError> {
    connect_db().await?;
    set_db().await?;
    define_tables().await?;
    define_starting_data().await
}

pub async fn define_tables() -> Result<(), ServerError> {
    define_providers().await?;
    add_default_providers().await?;

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
        define_chats(),
        define_users(),
        define_folders(),
    ]?;

    Ok(())
}

pub async fn define_starting_data() -> Result<(), ServerError> {
    let _ = tokio::try_join![add_all_ollama_models(), add_default_providers()]?;

    Ok(())
}
