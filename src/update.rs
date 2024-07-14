use iced::widget::combo_box;
use ollama_rs::Ollama;
use tokio::sync::Mutex as TMutex;
use std::sync::Arc;
use crate::chat::get_model;

pub struct Logic{
    pub models: combo_box::State<String>,
    pub chat: Option<usize>,
    pub ollama: Arc<TMutex<Ollama>>,
}

impl Logic{
    pub fn new() -> Self{
        Self{
            models: combo_box::State::new(Vec::new()),
            chat: None,
            ollama: Arc::new(TMutex::new(get_model())),
        }
    }
}
