use axum::{
    Router,
    routing::{get, post},
};

use crate::settings;

pub fn routes() -> Router {
    Router::new()
        .route(
            "/settings/",
            get(settings::get_settings).put(settings::update_settings),
        )
        .route("/settings/reset/", post(settings::reset_settings))
}
