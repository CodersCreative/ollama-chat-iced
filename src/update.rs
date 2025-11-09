use crate::{
    common::Id,
    providers::{Provider, SavedProviders},
};
use iced::widget::combo_box;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex as TMutex;

pub struct Logic {
    pub combo_models: combo_box::State<String>,
    pub models: Vec<String>,
    pub providers: HashMap<Id, Arc<TMutex<Provider>>>,
}

impl Logic {
    pub fn new(providers: &SavedProviders) -> Self {
        let mut map = HashMap::new();
        let mut models = Vec::new();

        for (key, provider) in providers.0.iter() {
            let new_provider: Provider = provider.into();
            models.append(&mut new_provider.get_models());
            map.insert(key.clone(), Arc::new(TMutex::new(new_provider)));
        }

        Self {
            combo_models: combo_box::State::new(models.clone()),
            models,
            providers: map,
        }
    }

    pub fn update_all_models(&mut self) {
        let mut models = Vec::new();
        for (_, provider) in self.providers.iter() {
            let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
            models.append(&mut tokio_runtime.block_on(async {
                let provider = provider.lock().await;
                provider.get_models_async().await
            }));
        }

        self.models = models.clone();
        self.combo_models = combo_box::State::new(models);
    }

    pub fn get_random_provider(&self) -> Option<Arc<TMutex<Provider>>> {
        // TODO remove everywhere this function is used!!!
        let mut val = None;
        for provider in self.providers.iter() {
            val = Some(provider.1.clone());
            break;
        }
        val
    }
}
