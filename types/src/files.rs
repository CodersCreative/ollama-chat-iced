use std::io::Read;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::surreal::RecordId;

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct B64FileData {
    pub b64data: String,
    #[serde(default = "FileType::default")]
    #[builder(default = "FileType::Image")]
    pub file_type: FileType,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum FileType {
    #[default]
    Image,
    Video,
    Audio,
    File,
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

impl TryInto<B64File> for DBFile {
    type Error = std::io::Error;
    fn try_into(self) -> Result<B64File, Self::Error> {
        let mut file = std::fs::File::open(self.path)?;
        let mut data = String::new();
        let _ = file.read_to_string(&mut data)?;

        Ok(B64File {
            b64data: data,
            file_type: self.file_type,
            id: self.id,
        })
    }
}
