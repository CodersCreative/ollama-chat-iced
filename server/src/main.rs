pub mod chats;
pub mod errors;
pub mod files;
pub mod generation;
pub mod options;
pub mod providers;
pub mod settings;
pub mod utils;

use chats::{messages, relationships};
use std::sync::LazyLock;
use surrealdb::{
    Surreal,
    engine::local::{Db, RocksDb},
};

static CONN: LazyLock<Surreal<Db>> = LazyLock::new(Surreal::init);

use axum::{
    Router,
    routing::{get, post, put},
};

use crate::{
    chats::{
        define_chats,
        previews::{self, define_previews},
        relationships::define_message_relationships,
    },
    errors::ServerError,
    files::define_files,
    messages::define_messages,
    options::{define_gen_options, relationships::define_gen_models},
    providers::{define_providers, ollama::models::define_ollama_models},
    settings::define_settings,
    utils::get_path_settings,
};

#[tokio::main]
async fn main() {
    init_db().await.unwrap();
    let app = Router::new()
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
            "/message/parent/{parent}/all/",
            get(messages::list_all_messages_from_parent),
        )
        .route("/message/all/", get(messages::list_all_messages))
        .route(
            "/message/{id}",
            get(messages::read_message)
                .put(messages::update_message)
                .delete(messages::delete_message),
        )
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
            "/settings/",
            get(settings::get_settings).put(settings::update_settings),
        )
        .route("/settings/reset/", post(settings::reset_settings))
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
        .route(
            "/provider/ollama/model/all/",
            get(providers::ollama::models::list_all_ollama_models),
        )
        .route(
            "/provider/ollama/model/search/{search}",
            get(providers::ollama::models::search_ollama_models),
        )
        .route(
            "/provider/{id}/model/{model}",
            post(providers::ollama::pull::run)
                .get(providers::models::get_provider_model)
                .delete(providers::models::delete_provider_model),
        )
        .route(
            "/provider/{id}/model/{model}/options/",
            get(options::relationships::get_default_gen_options_from_model),
        )
        .route(
            "/provider/{id}/model/all/",
            get(providers::models::list_all_provider_models),
        )
        .route("/provider/", post(providers::add_provider))
        .route("/provider/all/", get(providers::list_all_providers))
        .route(
            "/provider/{id}",
            get(providers::read_provider)
                .put(providers::update_provider)
                .delete(providers::delete_provider),
        )
        .route("/options/", post(options::add_gen_options))
        .route("/options/all/", get(options::list_all_gen_options))
        .route("/options/search/{search}", get(options::search_gen_options))
        .route(
            "/options/{id}",
            get(options::get_gen_options)
                .put(options::update_gen_options)
                .delete(options::delete_gen_options),
        )
        .route(
            "/options/relationship/",
            post(options::relationships::add_gen_models),
        )
        .route(
            "/options/relationship/all/",
            get(options::relationships::list_all_gen_models),
        )
        .route(
            "/options/relationship/{id}",
            get(options::relationships::get_gen_models)
                .put(options::relationships::update_gen_models)
                .delete(options::relationships::delete_gen_models),
        )
        .route(
            "/options/{id}/model/all/",
            get(options::relationships::get_models_from_options),
        )
        .route(
            "/options/{id}/all/",
            get(options::relationships::get_gen_models_from_options),
        )
        .route("/file/", post(files::create_file))
        .route("/file/all/", get(files::list_all_files))
        .route(
            "/file/{id}",
            get(files::get_file)
                .put(files::update_file)
                .delete(files::delete_file),
        )
        .route("/generation/text/run/", get(generation::text::run))
        .route(
            "/generation/text/stream/",
            get(generation::text::stream::run),
        );

    #[cfg(feature = "sound")]
    let app = app
        .route("/generation/speech/tts/run/", get(generation::tts::run))
        .route(
            "/generation/speech/tts/stream/",
            get(generation::tts::stream::run),
        );

    // TODO create stt endpoints
    #[cfg(feature = "voice")]
    let app = app.route("/generation/speech/stt/run/", get(generation::text::run));

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

    let _ = define_settings().await?;
    let _ = define_messages().await?;
    let _ = define_chats().await?;
    let _ = define_message_relationships().await?;
    let _ = define_ollama_models().await?;
    let _ = define_previews().await?;
    let _ = define_files().await?;
    let _ = define_gen_options().await?;
    let _ = define_gen_models().await?;
    define_providers().await
}
