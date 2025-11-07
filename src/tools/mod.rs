use std::{collections::HashMap, error::Error, fs::File, io::Read};

use derive_builder::Builder;
use ollama_rs::generation::tools::ToolInfo;
use serde::{Deserialize, Serialize};
use tantivy::{
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{Field, Schema, Value, STORED, TEXT},
    DocAddress, Index, IndexWriter, Score, TantivyDocument,
};

use crate::{common::Id, utils::get_path_settings};

pub const TOOLS_PATH: &str = "tools.json";

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct SavedTool {}

impl Into<ToolInfo> for SavedTool {
    fn into(self) -> ToolInfo {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct SavedTools {
    pub tools: HashMap<Id, SavedTool>,
    pub data: (Index, Vec<Field>, Schema),
}

impl PartialEq for SavedTools {
    fn eq(&self, other: &Self) -> bool {
        self.tools == other.tools
    }
}

impl Default for SavedTools {
    fn default() -> Self {
        Self::new(HashMap::new())
    }
}

impl SavedTools {
    pub fn save(&self, path: &str) {
        let path = get_path_settings(path.to_string());
        let writer = File::create(path);

        if let Ok(writer) = writer {
            let _ = serde_json::to_writer_pretty(writer, &self.tools);
        }
    }

    pub fn new(tools: HashMap<Id, SavedTool>) -> Self {
        Self {
            data: Self::into_index(&tools).unwrap(),
            tools,
        }
    }

    pub fn load(path: &str) -> Result<Self, String> {
        Ok(Self::new(Self::load_prompts(path)?))
    }

    async fn get_prompts_paths() -> Result<Vec<String>, String> {
        let files = rfd::AsyncFileDialog::new()
            .add_filter("Json", &["json"])
            .pick_files()
            .await;

        if let Some(files) = files {
            return Ok(files
                .iter()
                .map(|x| {
                    x.path()
                        .to_path_buf()
                        .into_os_string()
                        .into_string()
                        .unwrap()
                })
                .collect());
        }

        Err("Failed".to_string())
    }

    pub fn load_prompts(path: &str) -> Result<HashMap<Id, SavedTool>, String> {
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

    pub fn import_new_tools(&mut self, path: &str) -> Result<(), String> {
        let tools = Self::load_from_file(path)?;
        for tool in tools {
            self.tools.insert(Id::new(), tool);
        }
        let _ = self.set_search_data().map_err(|e| e.to_string());
        Ok(())
    }

    fn load_from_file(path: &str) -> Result<Vec<SavedTool>, String> {
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

    fn into_index(
        tools: &HashMap<Id, SavedTool>,
    ) -> Result<(Index, Vec<Field>, Schema), Box<dyn Error>> {
        let mut schema_builder = Schema::builder();
        let command = schema_builder.add_text_field("command", TEXT | STORED);
        // let content = schema_builder.add_text_field("content", TEXT);
        // let title = schema_builder.add_text_field("title", TEXT);
        let schema = schema_builder.build();

        let index = Index::create_in_ram(schema.clone());
        let mut index_writer: IndexWriter = index.writer(50_000_000)?;

        for (_, tool) in tools {
            /*index_writer.add_document(doc!(
                command => todo!()
            ))?;*/
        }

        index_writer.commit()?;

        Ok((index, vec![command /*, content, title*/], schema))
    }

    pub fn set_search_data(&mut self) -> Result<(), Box<dyn Error>> {
        self.data = Self::into_index(&self.tools)?;
        Ok(())
    }

    pub fn search<'a>(&'a self, input: &'a str) -> Result<Vec<SavedTool>, Box<dyn Error>> {
        if input.is_empty() || self.tools.len() < 6 {
            return Ok(self.tools.iter().map(|x| x.1.clone()).collect());
        }

        let reader = self.data.0.reader()?;
        let searcher = reader.searcher();

        let query_parser = QueryParser::for_index(&self.data.0, self.data.1.clone());
        let query = query_parser.parse_query(input)?;

        let top_docs: Vec<(Score, DocAddress)> =
            searcher.search(&query, &TopDocs::with_limit(5))?;

        let mut tools = Vec::new();

        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc::<TantivyDocument>(doc_address)?;

            let command: String = match retrieved_doc
                .get_first(self.data.1[0].clone())
                .map(|x| x.as_str())
            {
                Some(Some(x)) => x.to_string(),
                _ => continue,
            };

            /*if let Some(prompt) = self.tools.iter().find(|(_, x)| x.command == command) {
                prompts.push(prompt.1.clone());
            }*/
        }

        Ok(tools)
    }
}
