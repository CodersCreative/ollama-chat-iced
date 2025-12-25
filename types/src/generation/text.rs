use crate::chats::messages::{Message, Role};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ChatStreamResult {
    Idle,
    Err(String),
    Generating(ChatResponse),
    Generated(ChatResponse),
    Finished,
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

impl From<Message> for ChatQueryMessage {
    fn from(value: Message) -> Self {
        Self {
            text: value.content,
            files: value.files,
            role: value.role,
        }
    }
}

pub fn split_text_into_thinking(text: String) -> (String, Option<String>) {
    if text.contains("<think>") {
        let c = text.clone();
        let split = c.split_once("<think>").unwrap();
        let mut content = split.0.to_string();
        let think = if split.1.contains("</think>") {
            let split2 = split.1.rsplit_once("</think>").unwrap();
            content.push_str(split2.1);
            split2.0.to_string()
        } else {
            split.1.to_string()
        };

        (
            content.trim().to_string(),
            if !think.trim().is_empty() {
                Some(think.trim().to_string())
            } else {
                None
            },
        )
    } else {
        (text, None)
    }
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

impl Default for ChatResponse {
    fn default() -> Self {
        Self {
            role: Role::AI,
            content: String::new(),
            thinking: None,
            func_calls: Vec::new(),
        }
    }
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
    pub tools: Vec<String>,
    #[serde(default = "Vec::new")]
    #[builder(default = "Vec::new()")]
    pub messages: Vec<ChatQueryMessage>,
}
