use crate::backend::{errors::ServerError, user};
use axum::{
    Json, Router,
    routing::{get, post},
};
use ochat_types::ServerFeatures;

pub async fn version() -> Result<Json<String>, ServerError> {
    Ok(Json(env!("CARGO_PKG_VERSION").to_string()))
}

pub async fn features() -> Result<Json<Vec<ServerFeatures>>, ServerError> {
    #[allow(unused_mut)]
    let mut features = Vec::new();

    #[cfg(feature = "sound")]
    features.push(ServerFeatures::Sound);

    #[cfg(feature = "python")]
    features.push(ServerFeatures::Python);

    Ok(Json(features))
}

pub fn auth_routes() -> Router {
    Router::new()
        .route("/version/", get(version))
        .route("/features/", get(features))
        .route("/signin/", post(user::signin))
        .route("/signup/", post(user::signup))
        .route("/users/", get(user::list_all_users))
}

pub fn routes() -> Router {
    Router::new()
        .route("/user/{id}", get(user::get_user_from_id))
        .route("/user/", get(user::get_current_user))
}
