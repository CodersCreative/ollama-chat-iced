use crate::backend::{CONN, errors::ServerError, utils::get_file_uploads_path};
use axum::{Json, extract::Path};
use base64::{Engine, prelude::BASE64_STANDARD};
use ochat_types::{
    files::{B64File, B64FileData, DBFile, FileType},
    surreal::RecordId,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};

pub const FILE_TABLE: &str = "files";
pub const EMBEDDINGS_TABLE: &str = "embeddings";

pub mod route;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DBFileData {
    user_id: Option<RecordId>,
    path: String,
    file_type: FileType,
    filename: String,
}

impl DBFileData {
    pub async fn save_file(value: B64FileData) -> Result<Self, ServerError> {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let path = get_file_uploads_path(format!("{}.ochat", time));

        let mut file = fs::File::create(&path)?;

        match value.file_type {
            FileType::File => {
                let data = if !["md", "json"].contains(&value.filename.rsplit_once(".").unwrap().1)
                {
                    let tmp_path = get_file_uploads_path(format!("{}{}", time, value.filename));
                    let mut tmp_file = fs::File::create(&tmp_path)?;
                    tmp_file.write_all(&BASE64_STANDARD.decode(&value.b64data).unwrap())?;
                    let data = BASE64_STANDARD.encode(markdownify::convert(tmp_path.as_str())?);
                    fs::remove_file(&tmp_path)?;
                    data
                } else {
                    value.b64data
                };

                file.write_all(data.as_bytes())?;
            }
            _ => {
                file.write_all(value.b64data.as_bytes())?;
            }
        }

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
DEFINE TABLE IF NOT EXISTS {0} SCHEMAFULL
    PERMISSIONS FOR select, update, delete WHERE user_id = $auth.id FOR create FULL;
DEFINE FIELD IF NOT EXISTS user_id ON TABLE {0} TYPE record DEFAULT ALWAYS $auth.id;
DEFINE FIELD IF NOT EXISTS file_type ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS filename ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS path ON TABLE {0} TYPE string;

DEFINE TABLE IF NOT EXISTS {1} SCHEMAFULL
    PERMISSIONS FOR select, update, delete WHERE user_id = $auth.id FOR create FULL;
DEFINE FIELD IF NOT EXISTS user_id ON TABLE {1} TYPE record DEFAULT ALWAYS $auth.id;
DEFINE FIELD IF NOT EXISTS file_id ON TABLE {1} TYPE string;
",
            FILE_TABLE, EMBEDDINGS_TABLE,
        ))
        .await?;
    Ok(())
}

pub async fn create_file(
    Json(file): Json<B64FileData>,
) -> Result<Json<Option<DBFile>>, ServerError> {
    let file = DBFileData::save_file(file).await?;
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
    id: Path<String>,
    Json(file): Json<B64FileData>,
) -> Result<Json<Option<DBFile>>, ServerError> {
    if let Some(prev) = CONN
        .select::<Option<DBFile>>((FILE_TABLE, id.trim()))
        .await?
    {
        fs::remove_file(&prev.path)?;
    }

    let file = DBFileData::save_file(file).await?;
    Ok(Json(
        CONN.update((FILE_TABLE, id.trim())).content(file).await?,
    ))
}

pub async fn delete_file(id: Path<String>) -> Result<Json<Option<DBFile>>, ServerError> {
    let file: Option<DBFile> = CONN.delete((FILE_TABLE, id.trim())).await?;

    if let Some(file) = &file {
        fs::remove_file(&file.path)?;
    }

    Ok(Json(file))
}

pub async fn list_all_files() -> Result<Json<Vec<DBFile>>, ServerError> {
    Ok(Json(CONN.select(FILE_TABLE).await?))
}
