use crate::files;
use axum::{
    Router,
    routing::{get, post},
};

pub fn routes() -> Router {
    Router::new()
        .route("/file/", post(files::create_file))
        .route("/file/all/", get(files::list_all_files))
        .route(
            "/file/{id}",
            get(files::get_file)
                .put(files::update_file)
                .delete(files::delete_file),
        )
}
