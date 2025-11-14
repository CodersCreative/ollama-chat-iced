use crate::chats::messages::Role;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ChatStreamResult {
    Err(String),
    Generating(ChatResponse),
    Finished(ChatResponse),
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct ChatQueryMessage {
    pub text: String,
    pub files: Vec<String>,
    pub role: Role,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FunctionCall {
    pub name: String,
    pub result: Option<Value>,
    pub args: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatResponse {
    pub role: Role,
    pub content: String,
    pub thinking: Option<String>,
    pub func_calls: Vec<FunctionCall>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct ChatQueryData {
    pub provider: String,
    pub model: String,
    pub messages: Vec<ChatQueryMessage>,
}
