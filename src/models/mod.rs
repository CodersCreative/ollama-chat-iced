pub mod message;
pub mod view;
pub mod model;

use crate::utils::get_path_settings;
use model::{ModelInfo, TempInfo};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{error::Error, fs::File, io::Read};
use tantivy::collector::TopDocs;
use tantivy::index::Index;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, OwnedValue, Schema, STORED, TEXT};
use tantivy::{doc, DocAddress, IndexWriter, Score, TantivyDocument};

const MODELS_PATH: &str = "models.json";

#[derive(Debug, Clone)]
pub struct SavedModels {
    pub models: Vec<ModelInfo>,
    pub descriptions: HashMap<String, String>,
    pub data: (Index, Vec<Field>, Schema),
}

impl Into<SaveableModels> for SavedModels {
    fn into(self) -> SaveableModels {
        SaveableModels {
            models: self.models,
            descriptions: self.descriptions,
        }
    }
}

impl Into<SavedModels> for SaveableModels {
    fn into(self) -> SavedModels {
        SavedModels {
            models: self.models.clone(),
            descriptions: self.descriptions.clone(),
            data: self.into_index().unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct SaveableModels {
    pub models: Vec<ModelInfo>,
    pub descriptions: HashMap<String, String>,
}

impl SaveableModels {
    pub fn save(&self, path: &str) {
        let path = get_path_settings(path.to_string());
        let writer = File::create(path);

        if let Ok(writer) = writer {
            let _ = serde_json::to_writer(writer, &self);
        }
    }

    pub fn load(path: &str) -> Result<Self, String> {
        let path = get_path_settings(path.to_string());
        let reader = File::open(path);

        if let Ok(mut reader) = reader {
            let mut data = String::new();
            let _ = reader
                .read_to_string(&mut data)
                .map_err(|e| e.to_string())?;

            let de_data = serde_json::from_str(&data);

            return match de_data {
                Ok(x) => Ok(x),
                Err(e) => Err(e.to_string()),
            };
        }

        return Err("Failed to open file".to_string());
    }

    fn into_index<'a>(&'a self) -> Result<(Index, Vec<Field>, Schema), Box<dyn Error>> {
        let mut schema_builder = Schema::builder();
        let name = schema_builder.add_text_field("title", TEXT | STORED);
        let desc = schema_builder.add_text_field("desc", TEXT);
        let author = schema_builder.add_text_field("author", TEXT);
        let schema = schema_builder.build();

        let index = Index::create_in_ram(schema.clone());
        let mut index_writer: IndexWriter = index.writer(50_000_000)?;

        for model in &self.models {
            if let Some(d) = self.descriptions.get(model.name.as_str()) {
                index_writer.add_document(doc!(
                    name => model.name.clone(),
                    desc => d.as_str(),
                    author => model.author.clone()
                ))?;
            }
        }

        index_writer.commit()?;

        Ok((index, vec![name, desc, author], schema))
    }

    pub fn init() -> Result<Self, Box<dyn Error>> {
        if let Ok(x) = Self::load(MODELS_PATH) {
            return Ok(x);
        }

        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();

        let resp = tokio_runtime.block_on(reqwest::get(
            "https://raw.githubusercontent.com/Jeffser/Alpaca/main/src/available_models.json",
        ))?;
        let content = tokio_runtime.block_on(resp.text())?;
        let data: HashMap<String, TempInfo> = serde_json::from_str(&content)?;

        let models: Vec<ModelInfo> = data
            .into_iter()
            .map(|x| {
                let mut info: ModelInfo = x.1.into();
                info.name = x.0;
                info
            })
            .collect();

        let resp = tokio_runtime.block_on(reqwest::get("https://raw.githubusercontent.com/Jeffser/Alpaca/main/src/available_models_descriptions.py"))?;
        let content = tokio_runtime.block_on(resp.text())?;
        let descriptions = extract_model_description(&content)?;

        let models = Self {
            models,
            descriptions,
        };

        models.save(MODELS_PATH);

        Ok(models)
    }
}

impl SavedModels {
    pub fn save(&self, path: &str) {
        let saveable: SaveableModels = self.clone().into();
        saveable.save(path)
    }

    pub fn init() -> Result<Self, Box<dyn Error>> {
        let saveable: SaveableModels = SaveableModels::init()?;
        Ok(saveable.into())
    }

    pub fn load(path: &str) -> Result<Self, String> {
        let saveable: SaveableModels = SaveableModels::load(path)?;
        Ok(saveable.into())
    }
    pub fn search<'a>(&'a self, input: &'a str) -> Result<Vec<ModelInfo>, Box<dyn Error>> {
        if input.is_empty() {
            return Ok(self.models.clone());
        }

        let reader = self.data.0.reader()?;
        let searcher = reader.searcher();

        let query_parser = QueryParser::for_index(&self.data.0, self.data.1.clone());
        let query = query_parser.parse_query(input)?;

        let top_docs: Vec<(Score, DocAddress)> =
            searcher.search(&query, &TopDocs::with_limit(10))?;
        let mut models = Vec::new();

        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc::<TantivyDocument>(doc_address)?;

            let model: String = match retrieved_doc.get_first(self.data.1[0].clone()) {
                Some(OwnedValue::Str(x)) => x.clone(),
                _ => continue,
            };

            let model = self.models.iter().find(|x| x.name == model).unwrap();
            models.push(model.clone());
        }

        Ok(models)
    }
}

fn extract_model_description(python_code: &str) -> Result<HashMap<String, String>, String> {
    let mut result = HashMap::new();

    let code_without_prefix = python_code
        .trim()
        .trim_start_matches("descriptions = {")
        .trim_end_matches("}");

    let pairs: Vec<&str> = code_without_prefix.split(",\n").map(|s| s.trim()).collect();

    let key_regex = Regex::new(r"'([^']+)'").map_err(|e| e.to_string())?;
    let value_regex = Regex::new(r#"_\("([^"]+)"\)"#).map_err(|e| e.to_string())?;

    for pair in pairs {
        if pair.is_empty() {
            continue;
        }

        let key_capture = key_regex.captures(pair);
        let value_capture = value_regex.captures(pair);

        if let (Some(key_caps), Some(value_caps)) = (key_capture, value_capture) {
            let key = key_caps
                .get(1)
                .map_or("", |m| m.as_str())
                .trim()
                .to_string();
            let value = value_caps
                .get(1)
                .map_or("", |m| m.as_str())
                .trim()
                .to_string();
            result.insert(key, value);
        } else {
            return Err(format!("Failed to parse pair: {}", pair));
        }
    }

    Ok(result)
}
