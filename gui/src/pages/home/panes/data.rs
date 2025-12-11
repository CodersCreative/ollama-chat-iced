use std::collections::HashMap;

use ochat_types::{
    chats::messages::Message,
    options::{GenOptions, relationships::GenModelRelationship},
    prompts::Prompt,
    providers::{
        hf::HFModel,
        ollama::{OllamaModelsInfo, PullModelStreamResult},
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
pub struct DownloadsData(pub Vec<DownloadData>);

#[derive(Debug, Clone, Default)]
pub struct MessagesData(pub HashMap<String, Message>);

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
pub struct DownloadData {
    pub model: OllamaModelsInfo,
    pub progress: PullModelStreamResult,
}

impl ModelsData {
    pub async fn get_ollama(search: Option<String>) -> Self {
        let req = DATA.read().unwrap().to_request();

        Self {
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
                .unwrap_or_default(),
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
                .unwrap_or_default(),
        }
    }
}

impl OptionsData {
    pub async fn get_gen_models(search: Option<String>) -> Self {
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
            .unwrap_or_default();

        let mut value = Vec::new();

        for option in options {
            let models: Vec<GenModelRelationship> = req
                .make_request(
                    &format!("option/{}/all/", option.id.key().to_string()),
                    &(),
                    RequestType::Get,
                )
                .await
                .unwrap_or_default();

            value.push(OptionData {
                option,
                models: models.into_iter().map(|x| x.into()).collect(),
            });
        }

        Self(value)
    }
}

impl PromptsData {
    pub async fn get_prompts(search: Option<String>) -> Self {
        let req = DATA.read().unwrap().to_request();

        Self(
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
            .unwrap_or_default(),
        )
    }
}
