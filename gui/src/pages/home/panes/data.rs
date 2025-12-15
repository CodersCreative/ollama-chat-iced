use std::collections::HashMap;

use iced::widget::markdown;
use ochat_types::{
    chats::{Chat, messages::Message},
    files::B64File,
    options::{GenOptions, relationships::GenModelRelationship},
    prompts::Prompt,
    providers::{
        hf::{HFModel, HFPullModelStreamResult},
        ollama::{OllamaModelsInfo, OllamaPullModelStreamResult},
    },
    settings::SettingsProvider,
    surreal::RecordId,
};

use crate::{DATA, data::RequestType};

#[derive(Debug, Clone, Default)]
pub struct HomePaneSharedData {
    pub downloads: DownloadsData,
    pub models: ModelsData,
    pub tools: ToolsData,
    pub prompts: PromptsData,
    pub options: OptionsData,
    pub messages: MessagesData,
}

#[derive(Debug, Clone, Default)]
pub struct DownloadsData {
    pub ollama: HashMap<u32, OllamaModelsInfo>,
    pub hf: HashMap<u32, HFModel>,
}

#[derive(Debug, Clone, Default)]
pub struct MessagesData(pub HashMap<String, MessageMk>);

#[derive(Debug)]
pub struct MessageMk {
    pub content: markdown::Content,
    pub thinking: Option<markdown::Content>,
    pub files: Vec<B64File>,
    pub base: Message,
}

impl Clone for MessageMk {
    fn clone(&self) -> Self {
        Self {
            content: markdown::Content::parse(&self.base.content),
            files: self.files.clone(),
            thinking: self
                .base
                .thinking
                .clone()
                .map(|x| markdown::Content::parse(&x)),
            base: self.base.clone(),
        }
    }
}

impl MessageMk {
    pub async fn get(message: Message) -> Self {
        let files = message.files.clone();
        let mut message = Self {
            content: markdown::Content::parse(&message.content),
            files: Vec::new(),
            thinking: message
                .thinking
                .clone()
                .map(|x| markdown::Content::parse(&x)),
            base: message,
        };

        let req = DATA.read().unwrap().to_request();

        for file in files {
            if let Ok(Some(file)) = req
                .make_request::<Option<B64File>, ()>(
                    &format!("file/{}", file),
                    &(),
                    RequestType::Get,
                )
                .await
            {
                message.files.push(file);
            }
        }

        message
    }
}

impl MessagesData {
    pub fn push(&mut self, other: Vec<MessageMk>) -> Vec<String> {
        let mut ids = Vec::new();
        for v in other.into_iter() {
            ids.push(v.base.id.key().to_string());
            self.0.insert(v.base.id.key().to_string(), v);
        }

        ids
    }

    pub async fn load_chat(
        chat_id: String,
        path: Option<Vec<i8>>,
    ) -> Result<Vec<MessageMk>, String> {
        let req = DATA.read().unwrap().to_request();

        let chat: Chat = req
            .make_request(&format!("chat/{}", chat_id), &(), RequestType::Get)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(x) = chat.root {
            Self::load_chat_from_root(x, path).await
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn load_chat_from_root(
        root_msg: String,
        path: Option<Vec<i8>>,
    ) -> Result<Vec<MessageMk>, String> {
        let req = DATA.read().unwrap().to_request();

        let messages: Vec<Message> = if let Some(path) = path {
            req.make_request(
                &format!("message/parent/{}", root_msg),
                &path,
                RequestType::Get,
            )
            .await
            .map_err(|e| e.to_string())?
        } else {
            req.make_request(
                &format!("message/parent/{}/default/", root_msg),
                &(),
                RequestType::Get,
            )
            .await
            .map_err(|e| e.to_string())?
        };

        let mut message_mks = Vec::new();

        for msg in messages.into_iter() {
            message_mks.push(MessageMk::get(msg).await);
        }

        Ok(message_mks)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ModelsData {
    pub ollama: Vec<OllamaModelsInfo>,
    pub hf: Vec<HFModel>,
}

#[derive(Debug, Clone, Default)]
pub struct PromptsData(pub Vec<Prompt>);

#[derive(Debug, Clone, Default)]
pub struct ToolsData();

#[derive(Debug, Clone, Default)]
pub struct OptionsData(pub Vec<OptionData>);

#[derive(Debug, Clone)]
pub struct OptionData {
    pub option: GenOptions,
    pub models: Vec<OptionRelationshipData>,
}

#[derive(Debug, Clone)]
pub struct OptionRelationshipData {
    pub model: Option<SettingsProvider>,
    pub option: String,
    pub id: Option<RecordId>,
}

impl From<GenModelRelationship> for OptionRelationshipData {
    fn from(value: GenModelRelationship) -> Self {
        Self {
            model: Some(SettingsProvider {
                provider: value.provider,
                model: value.model,
            }),
            option: value.option,
            id: Some(value.id),
        }
    }
}

impl Into<GenModelRelationship> for OptionRelationshipData {
    fn into(self) -> GenModelRelationship {
        let model = self.model.unwrap();
        GenModelRelationship {
            provider: model.provider,
            model: model.model,
            option: self.option,
            id: self.id.unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OllamaDownloadData {
    pub model: OllamaModelsInfo,
    pub progress: OllamaPullModelStreamResult,
}

#[derive(Debug, Clone)]
pub struct HFDownloadData {
    pub model: HFModel,
    pub progress: HFPullModelStreamResult,
}

impl ModelsData {
    pub async fn get(search: Option<String>) -> Result<Self, String> {
        let req = DATA.read().unwrap().to_request();

        Ok(Self {
            ollama: req
                .make_request::<Vec<OllamaModelsInfo>, ()>(
                    &if let Some(search) = &search {
                        format!("provider/ollama/model/search/{}", search)
                    } else {
                        "provider/ollama/model/all/".to_string()
                    },
                    &(),
                    RequestType::Get,
                )
                .await
                .map(|x| {
                    if x.len() > 200 {
                        x[0..=200].to_vec()
                    } else {
                        x
                    }
                })
                .map_err(|e| e.to_string())?,
            hf: req
                .make_request::<Vec<HFModel>, ()>(
                    &if let Some(search) = &search {
                        format!("provider/hf/text/model/search/{}", search)
                    } else {
                        "provider/hf/text/model/all/".to_string()
                    },
                    &(),
                    RequestType::Get,
                )
                .await
                .map(|x| {
                    if x.len() > 100 {
                        x[0..=100].to_vec()
                    } else {
                        x
                    }
                })
                .map_err(|e| e.to_string())?,
        })
    }
}

impl OptionsData {
    pub async fn get(search: Option<String>) -> Result<Self, String> {
        let req = DATA.read().unwrap().to_request();

        let options: Vec<GenOptions> = req
            .make_request(
                &if let Some(search) = search {
                    format!("option/search/{}", search)
                } else {
                    "option/all/".to_string()
                },
                &(),
                RequestType::Get,
            )
            .await
            .map_err(|e| e.to_string())?;

        let mut value = Vec::new();

        for option in options {
            let models: Vec<GenModelRelationship> = req
                .make_request(
                    &format!("option/{}/all/", option.id.key().to_string()),
                    &(),
                    RequestType::Get,
                )
                .await
                .map_err(|e| e.to_string())?;

            value.push(OptionData {
                option,
                models: models.into_iter().map(|x| x.into()).collect(),
            });
        }

        Ok(Self(value))
    }
}

impl PromptsData {
    pub async fn get(search: Option<String>) -> Result<Self, String> {
        let req = DATA.read().unwrap().to_request();

        Ok(Self(
            req.make_request::<Vec<Prompt>, ()>(
                &if let Some(search) = search {
                    format!("prompt/search/{}", search)
                } else {
                    "prompt/all/".to_string()
                },
                &(),
                RequestType::Get,
            )
            .await
            .map_err(|e| e.to_string())?,
        ))
    }
}
