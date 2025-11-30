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
    #[serde(default = "Vec::new")]
    #[builder(default = "Vec::new()")]
    pub files: Vec<String>,
    #[serde(default = "Role::default")]
    #[builder(default = "Role::User")]
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
    #[serde(default = "Vec::new")]
    pub func_calls: Vec<FunctionCall>,
}

impl Into<ChatQueryMessage> for ChatResponse {
    fn into(self) -> ChatQueryMessage {
        ChatQueryMessageBuilder::default()
            .text(self.content)
            .role(self.role)
            .build()
            .unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct ChatQueryData {
    pub provider: String,
    pub model: String,
    #[serde(default = "Vec::new")]
    #[builder(default = "Vec::new()")]
    pub messages: Vec<ChatQueryMessage>,
}
