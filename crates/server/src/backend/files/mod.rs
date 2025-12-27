use crate::backend::{
    CONN,
    errors::ServerError,
    providers::{PROVIDER_TABLE, provider_into_config},
    settings::get_settings,
    utils::get_file_uploads_path,
};
use axum::{Json, extract::Path};
use base64::{Engine, prelude::BASE64_STANDARD};
use ochat_types::{
    files::{B64File, B64FileData, DBFile, FileType},
    providers::Provider,
    surreal::RecordId,
};
use rig::{client::EmbeddingsClient, embeddings::EmbeddingModel};
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

pub async fn generate_embeddings(
    converter: &transmutation::Converter,
    path: &str,
) -> Result<Vec<Vec<f64>>, ServerError> {
    let settings = get_settings().await?.0;
    if let Some(provider) = settings.embeddings_provider {
        let result = converter
            .convert(path)
            .to(transmutation::OutputFormat::EmbeddingReady {
                max_chunk_size: 512,
                overlap: 128,
            })
            .with_options(transmutation::ConversionOptions {
                optimize_for_llm: true,
                split_pages: false,
                ..Default::default()
            })
            .execute()
            .await
            .unwrap();

        let data: Vec<Vec<u8>> = result.content.into_iter().map(|x| x.data).collect();

        let request: Vec<String> = data
            .into_iter()
            .map(|x| String::from_utf8(x).unwrap_or_default())
            .collect();

        let model = provider.model.trim();
        let embeddings = if let Some(provider) = CONN
            .select::<Option<Provider>>((PROVIDER_TABLE, provider.provider.trim()))
            .await?
        {
            let provider = provider_into_config(&provider);
            provider.embedding_model(model).embed_texts(request).await?
        } else {
            panic!()
        };

        Ok(embeddings.into_iter().map(|x| x.vec).collect())
    } else {
        Err(ServerError::Unknown(String::from(
            "No default embeddings model set",
        )))
    }
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
                let mut tmp_path = get_file_uploads_path(format!("{}{}", time, value.filename));
                let mut tmp_file = fs::File::create(&tmp_path)?;
                tmp_file.write_all(&BASE64_STANDARD.decode(&value.b64data).unwrap())?;
                let converter = transmutation::Converter::new()?;
                if tmp_path.rsplit_once(".").unwrap().1 != "md" {
                    let result = converter
                        .convert(&tmp_path)
                        .to(transmutation::OutputFormat::Markdown {
                            split_pages: true,
                            optimize_for_llm: true,
                        })
                        .with_options(transmutation::ConversionOptions {
                            split_pages: true,
                            optimize_for_llm: true,
                            ..Default::default()
                        })
                        .execute()
                        .await?;
                    fs::remove_file(&tmp_path)?;
                    tmp_path = format!("{}.md", tmp_path.rsplit_once(".").unwrap().0);
                    result.save(&tmp_path).await?;
                }

                // Save the .md base64 data
                {
                    let data = fs::read_to_string(&tmp_path)?;
                    file.write_all(BASE64_STANDARD.encode(data).as_bytes())?;
                }

                let _embeddings = generate_embeddings(&converter, &tmp_path).await;

                fs::remove_file(&tmp_path)?;
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

DEFINE TABLE IF NOT EXISTS {0} SCHEMAFULL
    PERMISSIONS FOR select, update, delete WHERE user_id = $auth.id FOR create FULL;
DEFINE FIELD IF NOT EXISTS user_id ON TABLE {0} TYPE record DEFAULT ALWAYS $auth.id;
DEFINE FIELD IF NOT EXISTS file_id ON TABLE {0} TYPE string;
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
