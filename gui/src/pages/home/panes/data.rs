use std::collections::HashMap;

use ochat_types::{
    chats::messages::Message,
    options::{GenOptions, relationships::GenModelRelationship},
    prompts::Prompt,
    providers::ollama::{OllamaModelsInfo, PullModelStreamResult},
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
pub struct ModelsData(pub Vec<OllamaModelsInfo>);

#[derive(Debug, Clone, Default)]
pub struct PromptsData(pub Vec<Prompt>);

#[derive(Debug, Clone, Default)]
pub struct ToolsData();

#[derive(Debug, Clone, Default)]
pub struct OptionsData(pub Vec<OptionData>);

#[derive(Debug, Clone)]
pub struct OptionData {
    pub option: GenOptions,
    pub models: Vec<GenModelRelationship>,
}

#[derive(Debug, Clone)]
pub struct DownloadData {
    pub model: OllamaModelsInfo,
    pub progress: PullModelStreamResult,
}

impl ModelsData {
    pub async fn get_ollama(search: Option<String>) -> Self {
        let req = DATA.read().unwrap().to_request();

        Self(
            req.make_request::<Vec<OllamaModelsInfo>, ()>(
                &if let Some(search) = search {
                    format!("provider/ollama/model/search/{}", search)
                } else {
                    "provider/ollama/model/all/".to_string()
                },
                &(),
                RequestType::Get,
            )
            .await
            .map(|x| {
                if x.len() > 400 {
                    x[0..=400].to_vec()
                } else {
                    x
                }
            })
            .unwrap_or_default(),
        )
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

            value.push(OptionData { option, models });
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
