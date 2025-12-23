use axum::{
    Router,
    routing::{get, post},
};

use crate::backend::options;

pub fn routes() -> Router {
    Router::new()
        .route("/option/", post(options::add_gen_options))
        .route("/option/all/", get(options::list_all_gen_options))
        .route("/option/search/{search}", get(options::search_gen_options))
        .route(
            "/option/{id}",
            get(options::get_gen_options)
                .put(options::update_gen_options)
                .delete(options::delete_gen_options),
        )
        .route(
            "/option/relationship/",
            post(options::relationships::add_gen_models),
        )
        .route(
            "/option/relationship/all/",
            get(options::relationships::list_all_gen_models),
        )
        .route(
            "/option/relationship/{id}",
            get(options::relationships::get_gen_models)
                .put(options::relationships::update_gen_models)
                .delete(options::relationships::delete_gen_models),
        )
        .route(
            "/option/{id}/model/all/",
            get(options::relationships::get_models_from_options),
        )
        .route(
            "/option/{id}/all/",
            get(options::relationships::get_gen_models_from_options),
        )
}
