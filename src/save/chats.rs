use std::time::SystemTime;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Renderer, text_editor, button, column, combo_box, container, horizontal_space, image, row, scrollable, text, markdown, keyed_column, svg, scrollable::{Direction, Scrollbar}};
use iced::{Element, Length, Padding, Task, Theme};
use kalosm_sound::{rodio::buffer::SamplesBuffer, MicInput};
use ollama_rs::coordinator::Coordinator;
use serde::{Deserialize, Serialize};
use crate::llm::{get_model, run_ollama_tools, Tools};
use crate::chats::{Chats, State};
use crate::common::Id;
use crate::sound::{get_audio, transcribe};
use crate::start::{self, Section};
use crate::style;
use crate::utils::{change_alpha, generate_id, get_path_assets, lighten_colour, get_preview};
use crate::{ChatApp, Message, SAVE_FILE};
use std::{path::PathBuf, sync::Arc};
use ollama_rs::generation::chat::ChatMessage;
use super::chat::{Chat, ChatBuilder};

use tokio::sync::Mutex;
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SavedChats ( pub Vec<Chat>, pub Id, pub Vec<Tools>, pub SystemTime);



#[derive(Default, Debug)]
pub struct TooledOptions {
    pub chats : Vec<ChatMessage>,
    pub tools : Vec<Tools>,
} 



#[derive(Debug, Clone)]
pub enum ChatsMessage{
    Regenerate,
    Submit,
    ChangeModel(String),
    Action(text_editor::Action),
    ChangeStart(String),
    ChangeChat(usize),
    NewChat,
    Listen,
    Convert(Option<SamplesBuffer<f32>>),
    Listened(Result<String, String>),
    PickedImage(Result<Vec<PathBuf>, String>),
    PickImage,
    RemoveImage(PathBuf),
}

impl ChatsMessage{
    pub fn handle(&self, id : Id, app : &mut ChatApp) -> Task<Message>{
        match self{

            Self::Regenerate => {
                let index = Chats::get_index(app, id);

                let mut s_index = 0;
                let saved_id = app.main_view.get_chats()[index].get_saved_chat().clone();

                for (i, x) in app.save.chats.iter_mut().enumerate(){
                    if x.1 == saved_id{
                        x.0.remove(x.0.len() - 1);
                        s_index = i;
                        break;
                    }
                }
                app.main_view.update_chats(|chats| {
                    chats.iter_mut().filter(|x| x.get_saved_chat() == &saved_id).for_each(|x| {
                        x.update_markdown(|x| {x.remove(x.len() - 1);});
                    });
                });

                let option = app.options.get_create_model_options_index(app.main_view.get_chats()[index].get_model().to_string());
                app.main_view.add_chat_stream(crate::llm::ChatStream::new(app, saved_id, option, s_index));
                
                Task::none()
            },
            Self::Listen => {
                let index = Chats::get_index(app, id);
                let mic = MicInput::default();
                let stream = mic.stream();
                
                app.main_view.update_chat(index, |chat| chat.set_state(State::Listening));
                Task::perform(get_audio(stream), move |x| Message::Chats(ChatsMessage::Convert(x), id))
            },
            Self::Convert(x) => {
                let index = Chats::get_index(app, id);
                
                app.main_view.update_chat(index, |chat| chat.set_state(State::Generating));
                Task::perform(transcribe(x.clone()), move |x| Message::Chats(ChatsMessage::Listened(x), id))
            },
            Self::Listened(x) => {
                let index = Chats::get_index(app, id);
                app.main_view.update_chat(index, |chat| {
                    if let Ok(str) = x{
                        chat.set_content_text(str);
                    }

                    chat.set_state(State::Idle);
                }); 

                Task::none()
            },
            Self::RemoveImage(x) => {
                let index = Chats::get_index(app, id);

                app.main_view.update_chat(index, |chat| {
                    if let Ok(x) = chat.get_images().binary_search(&x){
                        chat.get_images_mut().remove(x);
                    }
                });

                Task::none()
            },
            Self::PickedImage(x) => {
                let index = Chats::get_index(app, id);
                if let Ok(x) = x{
                    let mut x = x.clone();
                    app.main_view.update_chat(index, |chat| chat.add_images(&mut x));
                }
                Task::none()
            }
            Self::ChangeModel(x) => {
                let index = Chats::get_index(app, id);
                app.main_view.update_chat(index, |chat| {
                    if chat.get_state() == &State::Idle{
                        chat.set_model(x.clone());
                        app.save.save(SAVE_FILE);
                        let _ = app.options.get_create_model_options_index(x.clone());
                    }
                });

                Task::none()
            },
            Self::ChangeStart(x) => {
                let index = Chats::get_index(app, id);
                app.main_view.update_chat(index, |chat| chat.set_start(x.clone()));
                Task::none()
            },
            Self::ChangeChat(x) => {
                let index = Chats::get_index(app, id);
                
                app.main_view.update_chat(index, |chat| {
                    if chat.get_state() == &State::Idle{
                        chat.set_saved_chat(app.save.chats[x.clone()].1);
                        chat.set_markdown(app.save.chats[*x].to_mk());
                        // app.logic.chat = Some(Chats::get_index(app, id));
                        app.save.save(SAVE_FILE);
                    }
                }); 

                Task::none()
            },
            Self::Action(x) => {
                let index = Chats::get_index(app, id);
                app.main_view.update_chat(index, |chat| chat.content_perform(x.clone()));
                Task::none()
            },
            Self::NewChat => {
                let chats = Chats::get_from_id(app, id);
                if chats.get_state() == &State::Idle{
                    return Self::new_chat(app, id)
                }
                Task::none()
            },
            Self::Submit => {
                let index = Chats::get_index(app, id);
                let chat = ChatBuilder::default().content(app.main_view.get_chats()[index].get_content_text()).images(app.main_view.get_chats()[index].get_images().clone()).build().unwrap();

                let mut s_index = 0;
                let saved_id = app.main_view.get_chats()[index].get_saved_chat().clone();

                for (i, x) in app.save.chats.iter_mut().enumerate(){
                    if x.1 == saved_id{
                        x.0.push(chat.clone());
                        s_index = i;
                        break;
                    }
                }
                app.main_view.update_chats(|chats| {
                    chats.iter_mut().filter(|x| x.get_saved_chat() == &saved_id).for_each(|x| {
                        x.add_markdown(Chat::generate_mk(chat.get_content()));
                    });
                });

                let option = app.options.get_create_model_options_index(app.main_view.get_chats()[index].get_model().to_string());
                
                if app.save.chats[s_index].2.is_empty(){
                    app.main_view.add_chat_stream(crate::llm::ChatStream::new(app, saved_id, option, s_index));
                }

                app.main_view.update_chat(index, |chat| {
                    if app.save.chats[s_index].2.is_empty(){
                        chat.set_state(State::Generating);

                    }else{
                        let tooled = TooledOptions{
                            chats : app.save.chats[s_index].get_chat_messages(),
                            // tools : app.save.chats[s_index].2.clone(),
                            tools : vec![Tools::DuckDuckGo]
                        };

                        chat.set_tools(Arc::new(tooled));

                        // return Task::perform(run_ollama_tools(app.main_view.chats[index].tools.clone(), app.options.0[option].clone(), app.logic.ollama.clone()), |x| Message::None)
                    }
                    
                    chat.set_images(Vec::new());

                });

                
                Task::none()
            },
            Self::PickImage => {
                ChatApp::pick_images(id)
            }
        }
    }
}


impl SavedChats{
    pub fn new() -> Self{
        Self(Vec::new(), Id::new(), Vec::new(), SystemTime::now())
    }

    pub fn to_mk(&self) -> Vec<Vec<markdown::Item>>{
        return self.0.iter().map(|x| Chat::generate_mk(&x.get_content())).collect();
    }

    pub fn new_with_chats_tools(chats : Vec<Chat>, tools : Vec<Tools>) -> Self{
        return Self(chats, Id::new(), tools, SystemTime::now());
    }

    pub fn new_with_chats(chats: Vec<Chat>) -> Self{
        return Self(chats, Id::new(), Vec::new(), SystemTime::now());
    }

    pub fn get_preview(&self) -> (String, SystemTime){
        return get_preview(self);
    }

    pub fn get_chat_messages(&self) -> Vec<ChatMessage>{
        self.0.iter().map(|x| {
            x.into()
        }).collect()
    }
}
