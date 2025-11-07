use std::{collections::HashMap, error::Error, fs::File, io::Read};

use derive_builder::Builder;
use ollama_rs::generation::tools::{ToolFunctionInfo, ToolInfo};
use schemars::json_schema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tantivy::{
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{Field, Schema, Value as TantivyValue, STORED, TEXT},
    DocAddress, Index, IndexWriter, Score, TantivyDocument,
};

use crate::{common::Id, utils::get_path_settings};

pub const TOOLS_PATH: &str = "tools.json";

pub fn get_builtins() -> HashMap<String, Box<dyn ToolExecutable>> {
    HashMap::new()
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct SavedTool {
    pub title: String,
    pub description: String,
    pub functions: Vec<SavedToolFunc>,
    pub python: Option<String>,
    pub lua: Option<String>,
    pub builtins: Vec<String>,
}

impl SavedTool {
    pub fn regen_functions(&mut self) {
        self.functions = Vec::new();
        let funcs: Vec<SavedToolFunc> = Into::<Vec<SavedToolFunc>>::into(&*self);
        self.functions = funcs;
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct SavedToolFunc {
    pub name: String,
    pub desc: String,
    pub tool_type: ToolType,
    pub params: Vec<(String, String, bool)>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ToolType {
    Python,
    Lua,
    #[default]
    Builtin,
}

impl Into<ToolFunctionInfo> for &SavedToolFunc {
    fn into(self) -> ToolFunctionInfo {
        let required: Vec<String> = self
            .params
            .iter()
            .filter(|x| x.2)
            .map(|x| x.0.to_string())
            .collect();

        let mut params = json_schema!({});
        for param in self.params.iter() {
            params.insert(param.0.clone(), json!({"type" : param.1.clone()}));
        }

        ToolFunctionInfo {
            name: self.name.clone(),
            description: self.desc.clone(),
            parameters: json_schema!({"type" : "object", "properties" : params, "required" : required}),
        }
    }
}

impl Into<Vec<SavedToolFunc>> for &SavedTool {
    fn into(self) -> Vec<SavedToolFunc> {
        if !self.functions.is_empty() {
            return self.functions.clone();
        }

        let mut funcs = if !self.builtins.is_empty() {
            let builtins = get_builtins();

            self.builtins
                .iter()
                .filter_map(|x| builtins.get(x.as_str()).map(|x| (&**x).into()))
                .collect()
        } else {
            Vec::new()
        };

        let mut py_funcs = if let Some(text) = self.python.clone() {
            let lines: Vec<&str> = text.split('\n').filter(|x| x.trim().is_empty()).collect();

            let mut funcs: Vec<(String, String)> = Vec::new();

            for i in 0..lines.len() {
                if lines[i].trim().starts_with("async") || lines[i].trim().starts_with("def") {
                    let mut func = lines[i].trim().to_string();
                    let mut index = i + 1;
                    while !func.ends_with(":") {
                        func.push_str(&format!("\n{}", lines[index].trim()));
                        index += 1;
                    }

                    if lines[index].starts_with("\"\"\"") {
                        let mut comment = lines[index].trim().to_string();

                        while !comment.ends_with("\"\"\"") {
                            comment.push_str(&format!("\n{}", lines[index].trim()));
                            index += 1;
                        }

                        funcs.push((func, comment));
                    }
                }
            }

            let funcs: Vec<(String, Vec<(String, Option<String>, bool)>, String)> = funcs
                .into_iter()
                .map(|func| {
                    let params = func
                        .0
                        .trim()
                        .trim_start_matches("async")
                        .trim()
                        .trim_start_matches("def")
                        .trim();

                    let params = params.split_once("(").unwrap();
                    let name = params.0.to_string();
                    let params = params
                        .1
                        .trim()
                        .trim_start_matches("self")
                        .trim()
                        .trim_start_matches(',')
                        .trim()
                        .rsplit_once(')')
                        .unwrap()
                        .0
                        .split(',');

                    let mut final_params: Vec<(String, Option<String>, bool)> = Vec::new();

                    for param in params {
                        let has_default = param.contains('=');

                        let param = if has_default {
                            param.rsplit_once('=').unwrap().0.to_string()
                        } else {
                            param.to_string()
                        };

                        if let Some(x) = param.split_once(':') {
                            final_params.push((
                                x.0.trim().to_string(),
                                Some(x.0.trim().to_string()),
                                has_default,
                            ));
                        } else {
                            final_params.push((param.trim().to_string(), None, !has_default));
                        }
                    }

                    (name, final_params, func.1)
                })
                .collect();

            funcs
                .into_iter()
                .map(|func| SavedToolFunc {
                    name: func.0,
                    desc: func.2,
                    tool_type: ToolType::Python,
                    params: func
                        .1
                        .into_iter()
                        .map(|x| (x.0, x.1.unwrap_or("str".to_string()), x.2))
                        .collect(),
                })
                .collect()
        } else {
            Vec::new()
        };

        funcs.append(&mut py_funcs);
        funcs
    }
}

impl Into<Vec<ToolFunctionInfo>> for &SavedTool {
    fn into(self) -> Vec<ToolFunctionInfo> {
        Into::<Vec<SavedToolFunc>>::into(self)
            .into_iter()
            .map(|x| (&x).into())
            .collect()
    }
}

pub trait ToolExecutable {
    fn run(&self, params: HashMap<String, Value>) -> String;
    fn description(&self) -> String;
    fn params(&self) -> Vec<(String, String)>;
    fn name(&self) -> String;
}

impl Into<SavedToolFunc> for &dyn ToolExecutable {
    fn into(self) -> SavedToolFunc {
        SavedToolFunc {
            name: self.name(),
            desc: self.description(),
            tool_type: ToolType::Builtin,
            params: self
                .params()
                .into_iter()
                .map(|x| (x.0, x.1, true))
                .collect(),
        }
    }
}

impl Into<Vec<ToolInfo>> for &SavedTool {
    fn into(self) -> Vec<ToolInfo> {
        Into::<Vec<ToolFunctionInfo>>::into(self)
            .into_iter()
            .map(|func| ToolInfo {
                tool_type: ollama_rs::generation::tools::ToolType::Function,
                function: func,
            })
            .collect()
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

    async fn _get_tools_paths() -> Result<Vec<String>, String> {
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
        let description = schema_builder.add_text_field("description", TEXT);
        let title = schema_builder.add_text_field("title", TEXT | STORED);
        let schema = schema_builder.build();

        let index = Index::create_in_ram(schema.clone());
        let mut index_writer: IndexWriter = index.writer(50_000_000)?;

        for (_, tool) in tools {
            index_writer.add_document(doc!(

description => tool.description.as_str(),            title => tool.title.replace("-", " ").replace("_", " "),
                        ))?;
        }

        index_writer.commit()?;

        Ok((index, vec![title, description], schema))
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

            let title: String = match retrieved_doc
                .get_first(self.data.1[0].clone())
                .map(|x| x.as_str())
            {
                Some(Some(x)) => x.to_string(),
                _ => continue,
            };

            if let Some(tool) = self.tools.iter().find(|(_, x)| x.title == title) {
                tools.push(tool.1.clone());
            }
        }

        Ok(tools)
    }
}
