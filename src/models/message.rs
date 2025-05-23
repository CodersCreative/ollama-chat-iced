use crate::{common::Id, ChatApp, Message};
use iced::Task;

#[derive(Debug, Clone)]
pub enum ModelsMessage {
    Expand(String),
    Search,
    Input(String),
}

impl ModelsMessage {
    pub fn handle(&self, key: Id, app: &mut ChatApp) -> Task<Message> {
        match self {
            Self::Expand(x) => {
                app.main_view.update_model(&key, |model| {
                    if let Some(model) = model {
                        if model.0 != Some(x.clone()) {
                            model.0 = Some(x.clone());
                        } else {
                            model.0 = None;
                        }
                    }
                });
                Task::none()
            }
            Self::Input(x) => {
                app.main_view.update_model(&key, |model| {
                    if let Some(model) = model {
                        model.1 = x.clone();
                        if let Ok(search) = app.model_info.search(&model.1) {
                            model.2 = search;
                        }
                    }
                });
                Task::none()
            }
            Self::Search => {
                app.main_view.update_model(&key, |model| {
                    if let Some(model) = model {
                        if let Ok(search) = app.model_info.search(&model.1) {
                            model.2 = search;
                        }
                    }
                });
                Task::none()
            }
        }
    }
}
