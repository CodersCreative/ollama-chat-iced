use ochat_types::providers::ollama::{OllamaModelsInfo, PullModelStreamResult};

use crate::{DATA, data::RequestType};

#[derive(Debug, Clone, Default)]
pub struct HomePaneData {
    pub downloads: DownloadsData,
    pub models: ModelsData,
    pub tools: ToolsData,
    pub prompts: PromptsData,
    pub options: OptionsData,
}

#[derive(Debug, Clone, Default)]
pub struct DownloadsData(pub Vec<DownloadData>);

#[derive(Debug, Clone, Default)]
pub struct ModelsData(pub Vec<OllamaModelsInfo>);

#[derive(Debug, Clone, Default)]
pub struct PromptsData();

#[derive(Debug, Clone, Default)]
pub struct ToolsData();

#[derive(Debug, Clone, Default)]
pub struct OptionsData();

#[derive(Debug, Clone)]
pub struct DownloadData {
    pub model: OllamaModelsInfo,
    pub progress: PullModelStreamResult,
}

impl ModelsData {
    pub async fn get_ollama() -> Self {
        let req = DATA.read().unwrap().to_request();

        Self(
            req.make_request("provider/ollama/model/all/", &(), RequestType::Get)
                .await
                .unwrap_or_default(),
        )
    }
}
