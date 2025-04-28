use iced::Task;

use crate::{common::Id, llm::delete_model, ChatApp, Message};

use super::{values::OptionKey, SETTINGS_FILE};

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
        let model = app.main_view.options().get(&key).unwrap().0.clone();

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
                app.main_view.update_option(&key, |option| {
                    if let Some(option) = option {
                        option.set_model(x.clone());
                    }
                });

                Task::none()
            }
            Self::DeleteModel => {
                let model = app
                    .main_view
                    .options()
                    .get(&key)
                    .unwrap()
                    .model()
                    .to_string();

                if let Ok(i) = app.logic.models.binary_search(&model) {
                    app.logic.models.remove(i);
                    if let Some(m) = app.logic.models.first() {
                        app.main_view.update_option(&key, |option| {
                            if let Some(option) = option {
                                option.set_model(m.clone());
                            }
                        });
                        return Task::perform(
                            delete_model(app.logic.ollama.clone(), model.clone()),
                            move |_| Message::None,
                        );
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
                        let mut value = option.num_value.unwrap();
                        value.0 = num;
                        option.num_value = Some(value);
                    } else {
                        option.temp = option.num_value.unwrap().0.to_string()
                    }
                });

                app.options.save(SETTINGS_FILE);

                Task::none()
            }
            Self::ResetOption(x) => {
                let (m_index, index) = get_indexes(&x);

                app.options.update_gen_option(m_index, index, |option| {
                    let mut value = option.num_value.unwrap();
                    value.0 = value.1;
                    option.num_value = Some(value);
                    option.temp = value.1.to_string();
                    option.bool_value = false;
                });

                app.options.save(SETTINGS_FILE);
                Task::none()
            }
            Self::ClickedOption(x) => {
                if let Some(y) = &app.main_view.options().get(&key).unwrap().1 {
                    if x == y {
                        app.main_view
                            .update_option(&key, |x| x.unwrap().set_key(None));
                        return Task::none();
                    }
                }

                app.main_view
                    .update_option(&key, |y| y.unwrap().set_key(Some(x.clone())));
                Task::none()
            }
        }
    }
}
