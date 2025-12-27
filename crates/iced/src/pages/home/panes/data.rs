use crate::{DATA, RequestType};
use base64_stream::base64::{Engine, prelude::BASE64_STANDARD};
use iced::widget::markdown;
use ochat_types::{
    chats::{
        Chat,
        messages::{Message, MessageCanChange},
    },
    files::{B64File, FileType},
    options::{
        GenOptions,
        relationships::{GenModelRelationship, GenModelRelationshipData},
    },
    prompts::Prompt,
    providers::{
        hf::{HFModel, HFPullModelStreamResult},
        ollama::{OllamaModelsInfo, OllamaPullModelStreamResult},
    },
    settings::SettingsProvider,
    surreal::RecordId,
};
use std::{collections::HashMap, hash::Hash};

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

#[derive(Debug, Clone, Hash)]
pub struct ViewFile {
    pub filename: String,
    pub data: ViewFileType,
    pub id: String,
}

#[derive(Debug, Clone, Hash)]
pub enum ViewFileType {
    Image(Vec<u8>),
    Document(DocumentViewFile),
    Misc,
}

#[derive(Debug, Clone)]
pub struct DocumentViewFile {
    pub mk: Vec<markdown::Item>,
    pub text: String,
}

impl Hash for DocumentViewFile {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.text.hash(state)
    }
}

impl From<B64File> for ViewFile {
    fn from(value: B64File) -> Self {
        Self {
            filename: value.filename,
            data: match value.file_type {
                FileType::Image => {
                    ViewFileType::Image(BASE64_STANDARD.decode(value.b64data).unwrap())
                }
                FileType::File => {
                    let data = BASE64_STANDARD.decode(value.b64data).unwrap();
                    let data = String::from_utf8_lossy(&data).to_string();
                    ViewFileType::Document(DocumentViewFile {
                        mk: markdown::parse(&data).collect(),
                        text: data,
                    })
                }
                _ => ViewFileType::Misc,
            },
            id: value.id.key().to_string(),
        }
    }
}

#[derive(Debug)]
pub struct MessageMk {
    pub content: markdown::Content,
    pub thinking: Option<markdown::Content>,
    pub files: Vec<ViewFile>,
    pub can_change: bool,
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
            can_change: self.can_change,
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
            can_change: false,
            base: message,
        };

        let req = DATA.read().unwrap().to_request();

        for file in files {
            if let Ok(Some(file)) = req
                .make_request::<Option<B64File>, ()>(
                    &format!("file/{}", file.trim()),
                    &(),
                    RequestType::Get,
                )
                .await
            {
                message.files.push(file.into());
            }
        }

        if let Ok(can_change) = req
            .make_request::<MessageCanChange, ()>(
                &format!("message/{}/change/", message.base.id.key().to_string()),
                &(),
                RequestType::Get,
            )
            .await
        {
            message.can_change = can_change.can_change;
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
    pub user_id: Option<RecordId>,
    pub model: Option<SettingsProvider>,
    pub option: String,
    pub id: Option<RecordId>,
}

impl PartialEq for OptionRelationshipData {
    fn eq(&self, other: &Self) -> bool {
        self.model == other.model && self.option == other.option
    }
}

impl From<GenModelRelationship> for OptionRelationshipData {
    fn from(value: GenModelRelationship) -> Self {
        Self {
            user_id: Some(value.user_id),
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
            user_id: self.user_id.unwrap(),
            provider: model.provider,
            model: model.model,
            option: self.option,
            id: self.id.unwrap(),
        }
    }
}

impl Into<GenModelRelationshipData> for OptionRelationshipData {
    fn into(self) -> GenModelRelationshipData {
        let model = self.model.unwrap();
        GenModelRelationshipData {
            user_id: self.user_id,
            provider: model.provider,
            model: model.model,
            option: self.option,
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
