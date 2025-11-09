use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestFunctionMessageArgs,
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Read, sync::Arc, time::SystemTime};
use tokio::sync::Mutex;

use crate::{
    chats::chat::Role,
    common::Id,
    providers::Provider,
    sidebar::chats::SideChats,
    utils::{get_path_settings, split_text_new_line},
};

#[derive(Debug, Clone)]
pub struct PreviewResponse {
    pub text: String,
    pub chat: Id,
}

pub const PREVIEWS_FILE: &str = "previews.json";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct SavedPreviews(pub HashMap<Id, SavedPreview>);

impl SavedPreviews {
    pub fn in_order(&self) -> Vec<(Id, &SavedPreview)> {
        let mut lst: Vec<(Id, &SavedPreview)> = self.0.iter().map(|x| (x.0.clone(), x.1)).collect();
        lst.sort_by(|a, b| b.1.time.cmp(&a.1.time));
        lst
    }

    pub fn get_side_chats(&mut self) -> SideChats {
        SideChats::new(
            self.in_order()
                .into_iter()
                .map(|x| (x.0, x.1.text.to_string(), x.1.time.clone()))
                .collect(),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SavedPreview {
    pub text: String,
    pub time: SystemTime,
}

impl SavedPreviews {
    pub fn save(&self, path: &str) {
        let path = get_path_settings(path.to_string());
        let writer = File::create(path);

        if let Ok(writer) = writer {
            let _ = serde_json::to_writer_pretty(writer, &self);
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
}

pub async fn generate_preview(
    mut chats: Vec<(String, Role)>,
    chat: Id,
    model: String,
    provider: Arc<Mutex<Provider>>,
) -> Result<PreviewResponse, String> {
    chats.push((
        String::from(
            "
### Task:
Generate a concise, 3 word title for the previous messages.
### Guidelines:
- The title should clearly represent the main theme or subject of the conversation.
- Write the title in the chat's primary language; default to English if multilingual.
- Prioritize accuracy over excessive creativity; keep it clear and simple.
- Return the title by itself and nothing more.      
        ",
        ),
        Role::System,
    ));

    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages(
            chats
                .into_iter()
                .map(|chat| match chat.1 {
                    Role::User => ChatCompletionRequestUserMessageArgs::default()
                        .content(chat.0)
                        .build()
                        .unwrap_or_default()
                        .into(),
                    Role::AI => ChatCompletionRequestAssistantMessageArgs::default()
                        .content(chat.0)
                        .build()
                        .unwrap_or_default()
                        .into(),
                    Role::System => ChatCompletionRequestSystemMessageArgs::default()
                        .content(chat.0)
                        .build()
                        .unwrap_or_default()
                        .into(),
                    Role::Function => ChatCompletionRequestFunctionMessageArgs::default()
                        .content(chat.0)
                        .build()
                        .unwrap_or_default()
                        .into(),
                })
                .collect::<Vec<ChatCompletionRequestMessage>>(),
        )
        .build()
        .map_err(|e| e.to_string())?;

    let response = {
        let provider = provider.lock().await;
        provider
            .client
            .chat()
            .create(request)
            .await
            .map_err(|x| x.to_string())?
    };

    let mut value = String::new();

    for choice in response.choices.iter() {
        value.push_str(&choice.message.content.clone().unwrap_or_default());
    }

    Ok(PreviewResponse {
        text: if value.is_empty() {
            String::from("New")
        } else {
            split_text_new_line(value)
        },
        chat,
    })
}
