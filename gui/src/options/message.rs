use super::{values::OptionKey, SETTINGS_FILE};
use crate::{common::Id, ChatApp, Message};
use iced::Task;

#[derive(Debug, Clone)]
pub enum OptionMessage {
    ChangeOptionNum((String, OptionKey)),
    SubmitOptionNum(OptionKey),
    ChangeOptionBool((bool, OptionKey)),
    ClickedOption(OptionKey),
    ResetOption(OptionKey),
    ChangeModel(String),
    DeleteModel,
}

impl OptionMessage {
    pub fn handle<'a>(&'a self, key: Id, app: &'a mut ChatApp) -> Task<Message> {
        let model = match app.main_view.options().get(&key) {
            Some(x) => x.0.clone(),
            None => return Task::none(),
        };

        let mut get_indexes = |x: &OptionKey| -> (usize, usize) {
            let m_index = app.options.get_create_model_options_index(model.clone());
            (m_index, app.options.0[m_index].get_key_index(x.clone()))
        };

        match self {
            Self::ChangeOptionBool(x) => {
                let (m_index, index) = get_indexes(&x.1);

                app.options
                    .update_gen_option(m_index, index, |option| option.bool_value = x.0);
                app.options.save(SETTINGS_FILE);
                Task::none()
            }
            Self::ChangeModel(x) => {
                app.options.get_create_model_options_index(x.clone());
                app.main_view.update_option(&key, |option| {
                    if let Some(option) = option {
                        option.set_model(x.clone());
                    }
                });

                Task::none()
            }
            Self::DeleteModel => {
                let model = match app.main_view.options().get(&key) {
                    Some(x) => x.model().to_string(),
                    None => return Task::none(),
                };

                if let Ok(i) = app.logic.models.binary_search(&model) {
                    app.logic.models.remove(i);
                    if let Some(m) = app.logic.models.first() {
                        app.main_view.update_option(&key, |option| {
                            if let Some(option) = option {
                                option.set_model(m.clone());
                            }
                        });
                        // TODO delete ollama models
                        /*return Task::perform(
                            delete_model(app.logic.ollama.clone(), model.clone()),
                            move |_| Message::None,
                        );*/
                    }
                }

                Task::none()
            }
            Self::ChangeOptionNum(x) => {
                let (m_index, index) = get_indexes(&x.1);
                app.options
                    .update_gen_option(m_index, index, |option| option.temp = x.0.clone());
                Task::none()
            }
            Self::SubmitOptionNum(x) => {
                let (m_index, index) = get_indexes(&x);

                app.options.update_gen_option(m_index, index, |option| {
                    if let Ok(num) = option.temp.parse::<f32>() {
                        if let Some(mut value) = option.num_value {
                            value.0 = num;
                            option.num_value = Some(value);
                        }
                    } else {
                        if let Some(value) = option.num_value {
                            option.temp = value.0.to_string();
                        }
                    }
                });

                app.options.save(SETTINGS_FILE);

                Task::none()
            }
            Self::ResetOption(x) => {
                let (m_index, index) = get_indexes(&x);

                app.options.update_gen_option(m_index, index, |option| {
                    if let Some(mut value) = option.num_value {
                        value.0 = value.1;
                        option.num_value = Some(value);
                        option.temp = value.1.to_string();
                        option.bool_value = false;
                    }
                });

                app.options.save(SETTINGS_FILE);
                Task::none()
            }
            Self::ClickedOption(x) => {
                if let Some(y) = &app.main_view.options().get(&key).unwrap().1 {
                    if x == y {
                        app.main_view.update_option(&key, |x| {
                            if let Some(x) = x {
                                x.set_key(None)
                            }
                        });
                        return Task::none();
                    }
                }

                app.main_view.update_option(&key, |y| {
                    if let Some(y) = y {
                        y.set_key(Some(x.clone()))
                    }
                });
                Task::none()
            }
        }
    }
}
