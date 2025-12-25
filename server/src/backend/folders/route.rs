use crate::backend::folders;
use axum::{
    Router,
    routing::{get, post, put},
};

pub fn routes() -> Router {
    Router::new()
        .route("/folder/", post(folders::create_folder))
        .route("/folder/all/", get(folders::list_all_folders))
        .route("/folder/search/{search}", get(folders::search_folders))
        .route(
            "/folder/{id}",
            get(folders::get_folder)
                .put(folders::update_folder)
                .delete(folders::delete_folder),
        )
        .route(
            "/folder/{id}/parent/{parent}",
            put(folders::set_folder_parent),
        )
        .route("/folder/{id}/parent/none", put(folders::unparent_folder))
        .route(
            "/folder/{id}/chat/{chat}",
            put(folders::add_folder_chat).delete(folders::remove_folder_chat),
        )
        .route("/folder/archive/chat/{chat}", put(folders::archive_chat))
        .route("/folder/fav/chat/{chat}", put(folders::fav_chat))
}
