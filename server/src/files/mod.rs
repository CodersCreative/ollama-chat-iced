use std::{
    fs,
    io::{Read, Write},
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{Json, extract::Path};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

use crate::{CONN, errors::ServerError, utils::get_file_uploads_path};

pub const FILE_TABLE: &str = "files";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum FileType {
    #[default]
    Image,
    Video,
    Audio,
    File,
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct B64FileData {
    b64data: String,
    #[serde(default = "FileType::default")]
    #[builder(default = "FileType::Image")]
    file_type: FileType,
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
struct DBFileData {
    path: String,
    file_type: FileType,
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
            file_type: value.file_type,
        })
    }
}

impl TryInto<B64File> for DBFile {
    type Error = ServerError;
    fn try_into(self) -> Result<B64File, Self::Error> {
        let mut file = fs::File::open(self.path)?;
        let mut data = String::new();
        let _ = file.read_to_string(&mut data)?;

        Ok(B64File {
            b64data: data,
            file_type: self.file_type,
            id: self.id,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DBFile {
    pub path: String,
    pub file_type: FileType,
    pub id: RecordId,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct B64File {
    pub b64data: String,
    pub file_type: FileType,
    pub id: RecordId,
}

pub async fn define_files() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS file_type ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS path ON TABLE {0} TYPE string;
",
            FILE_TABLE
        ))
        .await?;
    Ok(())
}

pub async fn create_file(
    Json(file): Json<B64FileData>,
) -> Result<Json<Option<DBFile>>, ServerError> {
    let file = DBFileData::try_from(file)?;
    let file = CONN.create(FILE_TABLE).content(file).await?;
    Ok(Json(file))
}

pub async fn get_file(id: Path<String>) -> Result<Json<Option<B64File>>, ServerError> {
    let file: Option<DBFile> = CONN.select((FILE_TABLE, &*id)).await?;

    let file = match file {
        Some(x) => Some(x.try_into()?),
        _ => None,
    };

    Ok(Json(file))
}

pub async fn update_file(
    id: Path<String>,
    Json(file): Json<B64FileData>,
) -> Result<Json<Option<DBFile>>, ServerError> {
    if let Some(prev) = CONN.select::<Option<DBFile>>((FILE_TABLE, &*id)).await? {
        let _ = fs::remove_file(&prev.path)?;
    }

    let file = DBFileData::try_from(file)?;
    let file = CONN.update((FILE_TABLE, &*id)).content(file).await?;
    Ok(Json(file))
}

pub async fn delete_file(id: Path<String>) -> Result<Json<Option<DBFile>>, ServerError> {
    let file: Option<DBFile> = CONN.delete((FILE_TABLE, &*id)).await?;

    if let Some(file) = &file {
        let _ = fs::remove_file(&file.path)?;
    }

    Ok(Json(file))
}

pub async fn list_all_files() -> Result<Json<Vec<DBFile>>, ServerError> {
    let file = CONN.select(FILE_TABLE).await?;
    Ok(Json(file))
}
