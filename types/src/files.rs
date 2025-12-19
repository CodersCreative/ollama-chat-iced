use std::{fmt::Display, io::Read};

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::surreal::RecordId;

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct B64FileData {
    #[builder(default = "None")]
    pub user_id: Option<String>,
    pub b64data: String,
    pub filename: String,
    #[serde(default = "FileType::default")]
    #[builder(default = "FileType::Image")]
    pub file_type: FileType,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub enum FileType {
    #[default]
    Image,
    Video,
    Audio,
    File,
}

impl Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Image => "image",
                Self::Video => "video",
                Self::Audio => "audio",
                Self::File => "file",
            }
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DBFile {
    pub user_id: String,
    pub path: String,
    pub filename: String,
    pub file_type: FileType,
    pub id: RecordId,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct B64File {
    pub user_id: String,
    pub b64data: String,
    pub filename: String,
    pub file_type: FileType,
    pub id: RecordId,
}

impl TryInto<B64File> for DBFile {
    type Error = std::io::Error;
    fn try_into(self) -> Result<B64File, Self::Error> {
        let mut file = std::fs::File::open(self.path)?;
        let mut data = String::new();
        let _ = file.read_to_string(&mut data)?;

        Ok(B64File {
            user_id: self.user_id,
            b64data: data,
            filename: self.filename,
            file_type: self.file_type,
            id: self.id,
        })
    }
}
