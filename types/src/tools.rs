use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fmt::Display};

use crate::{prompts::OpenWebUIUser, surreal::RecordId};

#[derive(Serialize, Deserialize, Clone, Debug, Builder, Default)]
pub struct ToolData {
    #[builder(default = "None")]
    pub user_id: Option<RecordId>,
    pub name: String,
    #[serde(default = "Default::default")]
    #[builder(default = "Default::default()")]
    pub tool_type: ToolType,
    #[serde(default = "Default::default")]
    #[builder(default = "Default::default()")]
    pub tools: Vec<ToolInformation>,
    pub content: String,
    pub user: Option<OpenWebUIUser>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder, Default)]
pub struct ToolInformation {
    #[serde(default = "Vec::new")]
    #[builder(default = "Vec::new()")]
    pub name: Vec<String>,
    #[builder(default = "None")]
    pub description: Option<String>,
    #[builder(default = "None")]
    pub parameters: Option<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ToolType {
    Python,
    #[default]
    Lua,
    Builtin,
}

impl ToolType {
    pub const ALL_USER: [Self; 2] = [Self::Python, Self::Lua];
    pub const ALL: [Self; 3] = [Self::Python, Self::Lua, Self::Builtin];
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder, Default)]
pub struct ToolParameters {
    pub r#type: String,
    pub properties: HashMap<String, ToolParameter>,
    pub required: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum ToolDataType {
    #[default]
    #[serde(rename = "string")]
    String,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "object")]
    Object,
    #[serde(rename = "array")]
    Array,
    #[serde(rename = "boolean")]
    Bool,
    #[serde(rename = "null")]
    Null,
}

impl Display for ToolDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::String => "String",
                Self::Number => "Number",
                Self::Object => "Object",
                Self::Array => "Array",
                Self::Bool => "Bool",
                Self::Null => "Null",
            }
        )
    }
}

impl ToolDataType {
    pub const ALL: [Self; 6] = [
        Self::String,
        Self::Number,
        Self::Object,
        Self::Array,
        Self::Bool,
        Self::Null,
    ];
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder, Default)]
pub struct ToolParameter {
    pub r#type: ToolDataType,
    #[serde(rename = "enum")]
    pub enum_options: Option<Vec<String>>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tool {
    pub user_id: RecordId,
    pub name: String,
    #[serde(default = "Default::default")]
    pub tool_type: ToolType,
    #[serde(default = "Default::default")]
    pub tools: Vec<ToolInformation>,
    pub content: String,
    pub user: Option<OpenWebUIUser>,
    pub id: RecordId,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OpenWebUITool {}
