use axum::{
    Router,
    routing::{get, post},
};

use crate::backend::{options, providers};

pub fn routes() -> Router {
    Router::new()
        .route(
            "/provider/ollama/model/all/",
            get(providers::ollama::models::list_all_ollama_models),
        )
        .route(
            "/provider/ollama/model/search/{search}",
            get(providers::ollama::models::search_ollama_models),
        )
        .route(
            "/provider/hf/model/{user}/{id}",
            get(providers::hf::fetch_model_details),
        )
        .route(
            "/provider/hf/model/{user}/{id}/{name}",
            post(providers::hf::pull::run),
        )
        .route(
            "/provider/hf/model/downloaded/",
            get(providers::hf::get_downloaded_hf_models),
        )
        .route(
            "/provider/hf/text/model/all/",
            get(providers::hf::text::list_all_models),
        )
        .route(
            "/provider/hf/text/model/downloaded/",
            get(providers::hf::text::list_all_downloaded_models),
        )
        .route(
            "/provider/hf/text/model/search/{search}",
            get(providers::hf::text::search_models),
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
}
