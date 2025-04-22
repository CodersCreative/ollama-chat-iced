use iced::widget::combo_box;
use ollama_rs::Ollama;
use tokio::sync::Mutex as TMutex;
use std::sync::Arc;
use crate::llm::get_model;

pub struct Logic{
    pub combo_models: combo_box::State<String>,
    pub models : Vec<String>,
    // pub chat: Option<usize>,
    pub ollama: Arc<TMutex<Ollama>>,
}

impl Logic{
    pub fn new() -> Self{
        let mut val = Self{
            combo_models: combo_box::State::new(Vec::new()),
            models : Vec::new(),
            // chat: None,
            ollama: Arc::new(TMutex::new(get_model())),
        };

        val.models = val.get_models();

        val
    }

    pub async fn get_models_async(&self) -> Vec<String>{
        let o = self.ollama.lock().await;
        return o.list_local_models().await.unwrap().iter().map(|x| x.name.clone()).collect::<Vec<String>>();
    }

    pub fn get_models(&self) -> Vec<String>{
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
        tokio_runtime.block_on(self.get_models_async())
    }
}
