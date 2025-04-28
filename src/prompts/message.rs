use std::path::{Path, PathBuf};

use iced::{widget::text_editor, Task};

use crate::{common::Id, ChatApp, Message};

use super::{view::{Edit, Prompts}, Prompt, PromptBuilder, SavedPrompts, PROMPTS_PATH};


#[derive(Debug, Clone)]
pub enum PromptsMessage {
    Expand(String),
    Add,
    Search,
    Upload,
    Uploaded(Result<Vec<String>, String>),
    Input(String),
    EditTitle(String),
    EditCommand(String),
    EditAction(text_editor::Action),
    EditSave,
}

impl PromptsMessage {
    fn save_reload_prompts(app : &mut ChatApp){
        app.prompts.save(PROMPTS_PATH);
        
        app.main_view.update_prompts(|prompts| {
            for prompt in prompts{
                prompt.1.prompts = app.prompts.search(&prompt.1.input).unwrap();
            }
        });
    }

    pub fn handle(&self, key: Id, app: &mut ChatApp) -> Task<Message> {
        match self {
            Self::Expand(x) => {
                app.main_view.update_prompt(&key, |prompt| {
                    if let Some(prompt) = prompt {
                        if prompt.expand != Some(x.clone()) {
                            prompt.expand = Some(x.clone());
                            let p = prompt.prompts.iter().find(|y| &y.command == x).unwrap();
                            prompt.edit = Edit::from(p.clone());
                        } else {
                            prompt.expand = None;
                        }
                    }
                });
                Task::none()
            }
            Self::Upload => {
                Task::perform(SavedPrompts::get_prompts_paths(), move |x| Message::Prompts(PromptsMessage::Uploaded(x), key))
            }
            Self::Uploaded(x) => {
                if let Ok(paths) = x{
                    for path in paths{
                        let _ =app.prompts.import_new_prompts(&path);
                    }
                    Self::save_reload_prompts(app);
                }
                Task::none()
            }
            Self::Add => {
                app.main_view.update_prompt(&key, |prompt| {
                    if let Some(prompt) = prompt {
                        app.prompts.prompts.insert(Id::new(), PromptBuilder::default().title(prompt.input.clone()).command(prompt.input.clone()).content(String::new()).build().unwrap());
                        prompt.input = String::new();
                    }
                });

                Self::save_reload_prompts(app);
                Task::none()
            }
            Self::Input(x) => {
                app.main_view.update_prompt(&key, |prompt| {
                    if let Some(prompt) = prompt {
                        prompt.input = x.clone();
                    }
                });
                Task::none()
            }
            Self::Search => {
                app.main_view.update_prompt(&key, |prompt| {
                    if let Some(prompt) = prompt {
                        prompt.prompts = app.prompts.search(&prompt.input).unwrap();
                    }
                });
                Task::none()
            }
            Self::EditAction(x) => {
                app.main_view.update_prompt(&key, |prompt| {
                    if let Some(prompt) = prompt {
                        prompt.edit.content.perform(x.clone());
                    }
                });
                Task::none()
            }
            Self::EditTitle(x) => {
                app.main_view.update_prompt(&key, |prompt| {
                    if let Some(prompt) = prompt {
                        prompt.edit.title = x.clone();
                    }
                });
                Task::none()
            }
            Self::EditCommand(x) => {
                app.main_view.update_prompt(&key, |prompt| {
                    if let Some(prompt) = prompt {
                        prompt.edit.command = x.clone();
                    }
                });
                Task::none()
            }
            Self::EditSave => {
                if let Some(prompt) = app.main_view.prompts().get(&key){
                    if let Some(p) = app.prompts.prompts.iter_mut().find(|(i, x)| x.command == prompt.edit.og_command){
                        *p.1 = Prompt::from(&prompt.edit);
                    } 
                }

                Self::save_reload_prompts(app);
                Task::none()
            }
        }
    }

}
