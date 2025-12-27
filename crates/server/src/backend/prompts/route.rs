use crate::backend::prompts;
use axum::{
    Router,
    routing::{get, post},
};

pub fn routes() -> Router {
    Router::new()
        .route("/prompt/", post(prompts::add_prompt))
        .route("/prompt/all/", get(prompts::list_all_prompts))
        .route("/prompt/search/{search}", get(prompts::search_prompts))
        .route(
            "/prompt/{id}",
            get(prompts::get_prompt)
                .put(prompts::update_prompt)
                .delete(prompts::delete_prompt),
        )
}
