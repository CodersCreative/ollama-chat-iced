use std::{
    fs,
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{Extension, Json, extract::Path};
use ochat_types::{
    files::{B64File, B64FileData, DBFile, FileType},
    user::User,
};
use serde::{Deserialize, Serialize};

use crate::{CONN, errors::ServerError, utils::get_file_uploads_path};

pub const FILE_TABLE: &str = "files";

pub mod route;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DBFileData {
    user_id: Option<String>,
    path: String,
    file_type: FileType,
    filename: String,
}

impl TryFrom<B64FileData> for DBFileData {
    type Error = ServerError;
    fn try_from(value: B64FileData) -> Result<Self, Self::Error> {
        let path = get_file_uploads_path(format!(
            "{}.ochat",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        let mut file = fs::File::create(&path)?;
        let _ = file.write_all(value.b64data.as_bytes())?;

        Ok(Self {
            path,
            user_id: value.user_id,
            file_type: value.file_type,
            filename: value.filename,
        })
    }
}

pub async fn define_files() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS user_id ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS file_type ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS filename ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS path ON TABLE {0} TYPE string;
",
            FILE_TABLE
        ))
        .await?;
    Ok(())
}

pub async fn create_file(
    Extension(user): Extension<User>,
    Json(file): Json<B64FileData>,
) -> Result<Json<Option<DBFile>>, ServerError> {
    let mut file = DBFileData::try_from(file)?;
    file.user_id = Some(user.id.key().to_string());
    Ok(Json(CONN.create(FILE_TABLE).content(file).await?))
}

pub async fn get_file(id: Path<String>) -> Result<Json<Option<B64File>>, ServerError> {
    let file: Option<DBFile> = CONN.select((FILE_TABLE, id.trim())).await?;
    Ok(Json(match file {
        Some(x) => Some(x.try_into()?),
        _ => None,
    }))
}

pub async fn update_file(
    Extension(user): Extension<User>,
    id: Path<String>,
    Json(mut file): Json<B64FileData>,
) -> Result<Json<Option<DBFile>>, ServerError> {
    file.user_id = Some(user.id.key().to_string());
    if let Some(prev) = CONN
        .select::<Option<DBFile>>((FILE_TABLE, id.trim()))
        .await?
    {
        let _ = fs::remove_file(&prev.path)?;
    }

    let file = DBFileData::try_from(file)?;
    Ok(Json(
        CONN.update((FILE_TABLE, id.trim())).content(file).await?,
    ))
}

pub async fn delete_file(id: Path<String>) -> Result<Json<Option<DBFile>>, ServerError> {
    let file: Option<DBFile> = CONN.delete((FILE_TABLE, id.trim())).await?;

    if let Some(file) = &file {
        let _ = fs::remove_file(&file.path)?;
    }

    Ok(Json(file))
}

pub async fn list_all_files() -> Result<Json<Vec<DBFile>>, ServerError> {
    Ok(Json(CONN.select(FILE_TABLE).await?))
}
