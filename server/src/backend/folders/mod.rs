pub mod route;

use crate::backend::{CONN, errors::ServerError};
use axum::{Json, extract::Path};
use ochat_types::folders::{Folder, FolderData};

const FOLDER_TABLE: &str = "folders";

pub async fn define_folders() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMAFULL
    PERMISSIONS FOR select, update WHERE user_id = $auth.id FOR create FULL FOR delete WHERE name != 'Archived' and name != 'Favourites';
DEFINE FIELD IF NOT EXISTS user_id ON TABLE {0} TYPE record DEFAULT ALWAYS $auth.id;
DEFINE FIELD IF NOT EXISTS chats ON TABLE {0} TYPE array<string>;
DEFINE FIELD IF NOT EXISTS parent ON TABLE {0} TYPE option<string>;
DEFINE FIELD IF NOT EXISTS name ON TABLE {0} TYPE string;

DEFINE ANALYZER folders_analyzer TOKENIZERS class, blank FILTERS lowercase, ascii;
DEFINE INDEX name_index ON TABLE {0} COLUMNS name SEARCH ANALYZER folders_analyzer BM25;
",
            FOLDER_TABLE,
        ))
        .await?;
    Ok(())
}

pub async fn create_default_user_folders() -> Result<(), ServerError> {
    let _ = create_folder(Json(FolderData {
        user_id: None,
        chats: Vec::new(),
        parent: None,
        name: String::from("Archived"),
    }))
    .await?;
    let _ = create_folder(Json(FolderData {
        user_id: None,
        chats: Vec::new(),
        parent: None,
        name: String::from("Favourites"),
    }))
    .await?;
    Ok(())
}

pub async fn set_folder_parent(
    Path((id, parent)): Path<(String, String)>,
) -> Result<Json<Option<Folder>>, ServerError> {
    Ok(Json(
        CONN.query(&format!(
            "UPDATE {0}:{1} SET parent = '{2}';",
            FOLDER_TABLE,
            id.trim(),
            parent.trim()
        ))
        .await?
        .take(0)?,
    ))
}

pub async fn add_folder_chat(
    Path((id, chat)): Path<(String, String)>,
) -> Result<Json<Option<Folder>>, ServerError> {
    let _ = CONN
        .query(&format!(
            "UPDATE {0} SET chats -= '{1}' WHERE name != 'Favourites';",
            FOLDER_TABLE,
            chat.trim()
        ))
        .await?;

    Ok(Json(
        CONN.query(&format!(
            "UPDATE {0}:{1} SET chats += '{2}';",
            FOLDER_TABLE,
            id.trim(),
            chat.trim()
        ))
        .await?
        .take(0)?,
    ))
}

pub async fn remove_folder_chat(
    Path((id, chat)): Path<(String, String)>,
) -> Result<Json<Option<Folder>>, ServerError> {
    Ok(Json(
        CONN.query(&format!(
            "UPDATE {0}:{1} SET chats -= '{2}';",
            FOLDER_TABLE,
            id.trim(),
            chat.trim()
        ))
        .await?
        .take(0)?,
    ))
}

pub async fn unparent_folder(Path(id): Path<String>) -> Result<Json<Option<Folder>>, ServerError> {
    Ok(Json(
        CONN.query(&format!(
            "UPDATE {0}:{1} SET parent = NONE;",
            FOLDER_TABLE,
            id.trim(),
        ))
        .await?
        .take(0)?,
    ))
}

pub async fn fav_chat(Path(chat): Path<String>) -> Result<Json<Option<Folder>>, ServerError> {
    let folder = get_folder_from_name("Favourites").await?.unwrap();
    Ok(Json(
        CONN.query(&format!(
            "UPDATE {0}:{1} SET chats += '{2}';",
            FOLDER_TABLE,
            folder.id.key(),
            chat.trim()
        ))
        .await?
        .take(0)?,
    ))
}
pub async fn archive_chat(Path(chat): Path<String>) -> Result<Json<Option<Folder>>, ServerError> {
    let _ = CONN
        .query(&format!(
            "UPDATE {0} SET chats -= '{1}';",
            FOLDER_TABLE,
            chat.trim()
        ))
        .await?;

    let folder = get_folder_from_name("Archived").await?.unwrap();

    Ok(Json(
        CONN.query(&format!(
            "UPDATE {0}:{1} SET chats += '{2}';",
            FOLDER_TABLE,
            folder.id.key(),
            chat.trim()
        ))
        .await?
        .take(0)?,
    ))
}
pub async fn get_folder_from_name(name: &str) -> Result<Option<Folder>, ServerError> {
    let mut folder: Vec<Folder> = CONN
        .query(&format!(
            "            
SELECT * FROM {0} WHERE name = '{1}';
",
            FOLDER_TABLE, name
        ))
        .await?
        .take(0)?;

    if folder.is_empty() {
        Ok(None)
    } else {
        Ok(folder.pop())
    }
}
pub async fn create_folder(
    Json(folder): Json<FolderData>,
) -> Result<Json<Option<Folder>>, ServerError> {
    Ok(Json(CONN.create(FOLDER_TABLE).content(folder).await?))
}

pub async fn get_folder(id: Path<String>) -> Result<Json<Option<Folder>>, ServerError> {
    Ok(Json(CONN.select((FOLDER_TABLE, id.trim())).await?))
}

pub async fn update_folder(
    id: Path<String>,
    Json(folder): Json<FolderData>,
) -> Result<Json<Option<Folder>>, ServerError> {
    Ok(Json(
        CONN.update((FOLDER_TABLE, id.trim()))
            .content(folder)
            .await?,
    ))
}

pub async fn delete_folder(id: Path<String>) -> Result<Json<Option<Folder>>, ServerError> {
    Ok(Json(CONN.delete((FOLDER_TABLE, id.trim())).await?))
}

pub async fn list_all_folders() -> Result<Json<Vec<Folder>>, ServerError> {
    Ok(Json(CONN.select(FOLDER_TABLE).await?))
}

pub async fn search_folders(search: Path<String>) -> Result<Json<Vec<Folder>>, ServerError> {
    Ok(Json(
        CONN.query(&format!(
            "            
SELECT *, search::score(1) AS score FROM {0} WHERE name @1@ '{1}' ORDER BY score DESC;
",
            FOLDER_TABLE,
            search.trim()
        ))
        .await?
        .take(0)?,
    ))
}
