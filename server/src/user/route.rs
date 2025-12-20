use crate::{errors::ServerError, user};
use axum::{
    Json, Router,
    routing::{get, post},
};

pub async fn version() -> Result<Json<String>, ServerError> {
    Ok(Json(env!("CARGO_PKG_VERSION").to_string()))
}
pub fn auth_routes() -> Router {
    Router::new()
        .route("/version/", get(version))
        .route("/signin/", post(user::signin))
        .route("/signup/", post(user::signup))
        .route("/users/", get(user::list_all_users))
}

pub fn routes() -> Router {
    Router::new()
        .route("/user/{id}", get(user::get_user_from_id))
        .route("/user/", get(user::get_current_user))
}
