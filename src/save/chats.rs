use std::time::SystemTime;

use iced::alignment::Horizontal;
use iced::alignment::Vertical;
use iced::padding;
use iced::widget::text_editor;
use iced::widget::{button, column, combo_box, container, horizontal_space, image, row, scrollable, text, markdown, keyed_column, text_input, svg};
use iced::widget::scrollable::Direction;
use iced::widget::scrollable::Scrollbar;
use iced::Element;
use iced::Length;
use iced::Padding;
use iced::Task;
use iced::Theme;
use iced::widget::Renderer;
use kalosm_sound::rodio::buffer::SamplesBuffer;
use kalosm_sound::MicInput;
use serde::{Deserialize, Serialize};
use crate::sound::get_audio;
use crate::sound::transcribe;
use crate::start;
use crate::start::Section;
use crate::style;
use crate::utils::change_alpha;
use crate::utils::generate_id;
use crate::utils::get_path_assets;
use crate::utils::lighten_colour;
use crate::ChatApp;
use crate::SAVE_FILE;
use crate::{utils::get_preview, Message};
use std::{path::PathBuf, sync::Arc};

use ollama_rs::generation::chat::ChatMessage;
use super::chat::Chat;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SavedChats ( pub Vec<Chat>, pub i32, pub SystemTime);



#[derive(Debug)]
pub struct Chats {
    pub markdown: Vec<Vec<markdown::Item>>,
    pub images: Vec<PathBuf>,
    pub gen_chats : Arc<Vec<ChatMessage>>,
    pub state : State,
    pub start : String,
    pub input : text_editor::Content,
    input_height: f32,
    pub saved_id : i32,
    pub model : String,
    pub id : i32,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum State{
    Generating,
    Listening,
    Idle,
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
    PickedImage(Result<Vec<PathBuf>, String>),
    Listened(Result<String, String>),
    PickImage,
    RemoveImage(PathBuf),
}

impl ChatsMessage{
    pub fn handle(&self, id : i32, app : &mut ChatApp) -> Task<Message>{
        match self{

            Self::Regenerate => {
                let index = Chats::get_index(app, id);

                let mut s_index = 0;
                let saved_id = app.main_view.chats[index].saved_id;

                for (i, x) in app.save.chats.iter_mut().enumerate(){
                    if x.1 == saved_id{
                        x.0.remove(x.0.len() - 1);
                        s_index = i;
                        break;
                    }
                }

                app.main_view.chats.iter_mut().filter(|x| x.saved_id == saved_id).for_each(|x| {
                    x.markdown.remove(x.markdown.len() - 1);
                });

                let option = app.options.get_create_model_options_index(app.main_view.chats[index].model.clone());
                app.main_view.chat_streams.push(crate::chat::ChatStream::new(app, saved_id, option, s_index));

                Task::none()
            },
            Self::Listen => {
                let index = Chats::get_index(app, id);
                let mic = MicInput::default();
                let stream = mic.stream();
                
                app.main_view.chats[index].state = State::Listening;
                Task::perform(get_audio(stream), move |x| Message::Chats(ChatsMessage::Convert(x), id))
            },
            Self::Convert(x) => {
                let index = Chats::get_index(app, id);
                
                app.main_view.chats[index].state = State::Generating;
                Task::perform(transcribe(x.clone()), move |x| Message::Chats(ChatsMessage::Listened(x), id))
            },
            Self::Listened(x) => {
                let index = Chats::get_index(app, id);
                
                if let Ok(str) = x{
                    app.main_view.chats[index].input = text_editor::Content::with_text(str);
                }

                app.main_view.chats[index].state = State::Idle;
                Task::none()
            },
            Self::RemoveImage(x) => {
                let index = Chats::get_index(app, id);
                if let Ok(x) = app.main_view.chats[index].images.binary_search(&x){
                    app.main_view.chats[index].images.remove(x);
                }
                Task::none()
            },
            Self::PickedImage(x) => {
                let index = Chats::get_index(app, id);
                if let Ok(x) = x{
                    let mut x = x.clone();
                    app.main_view.chats[index].images.append(&mut x);
                }
                Task::none()
            }
            Self::ChangeModel(x) => {
                let index = Chats::get_index(app, id);
                if app.main_view.chats[index].state == State::Idle{
                    app.main_view.chats[index].model = x.clone();
                    app.save.save(SAVE_FILE);
                    let _ = app.options.get_create_model_options_index(x.clone());
                }
                Task::none()
            },
            Self::ChangeStart(x) => {
                let index = Chats::get_index(app, id);
                app.main_view.chats[index].start = x.clone();
                Task::none()
            },
            Self::ChangeChat(x) => {
                let index = Chats::get_index(app, id);
                
                if app.main_view.chats[index].state == State::Idle{
                    app.main_view.chats[index].saved_id = app.save.chats[x.clone()].1;
                    app.main_view.chats[index].markdown = app.save.chats[*x].to_mk();
                    app.logic.chat = Some(Chats::get_index(app, id));
                    app.save.save(SAVE_FILE);
                }

                Task::none()
            },
            Self::Action(x) => {
                let index = Chats::get_index(app, id);
                app.main_view.chats[index].input.perform(x.clone());
                Task::none()
            },
            Self::NewChat => {
                let chats = Chats::get_from_id(app, id);
                if chats.state == State::Idle{
                    return Self::new_chat(app, id)
                }
                Task::none()
            },
            Self::Submit => {
                let index = Chats::get_index(app, id);
                let chat = Chat{
                    role: super::chat::Role::User,
                    message: app.main_view.chats[index].input.text(),
                    images: app.main_view.chats[index].images.clone(), 
                };


                let mut s_index = 0;
                let saved_id = app.main_view.chats[index].saved_id;

                for (i, x) in app.save.chats.iter_mut().enumerate(){
                    if x.1 == saved_id{
                        x.0.push(chat.clone());
                        s_index = i;
                        break;
                    }
                }

                app.main_view.chats.iter_mut().filter(|x| x.saved_id == saved_id).for_each(|x| {
                    x.markdown.push(Chat::generate_mk(&chat.message.clone()));
                });

                let option = app.options.get_create_model_options_index(app.main_view.chats[index].model.clone());
                app.main_view.chat_streams.push(crate::chat::ChatStream::new(app, saved_id, option, s_index));

                app.main_view.chats[index].state = State::Generating;
                app.main_view.chats[index].images = Vec::new();
                Task::none()
            },
            Self::PickImage => {
                ChatApp::pick_images(id)
            }
        }
    }
}

impl Clone for Chats{
    fn clone(&self) -> Self {
        Self::new(self.model.clone(), self.saved_id, self.markdown.clone())
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone();
    }
}

impl Chats{
    pub fn new(model : String, saved_id : i32, markdown : Vec<Vec<markdown::Item>>) -> Self{
        Self{
            id: generate_id(),
            model,
            saved_id,
            markdown,
            start : "General".to_string(),
            state : State::Idle,
            input: text_editor::Content::new(),
            input_height: 50.0,
            images: Vec::new(),
            gen_chats: Arc::new(Vec::new()),
        }
    }

    pub fn get_from_id<'a>(app: &'a ChatApp, id : i32) -> &'a Self{
        app.main_view.chats.iter().find(|x| x.id == id).unwrap()
    }


    pub fn get_from_id_mut<'a>(app: &'a mut ChatApp, id : i32) -> &'a mut Self{
        app.main_view.chats.iter_mut().find(|x| x.id == id).unwrap()
    }

    pub fn get_index<'a>(app : &'a ChatApp, id : i32) -> usize{
        for i in 0..app.main_view.chats.len(){
            if app.main_view.chats[i].id == id{
                return i
            }
        }
        0
    }

    pub fn get_saved_index(&self, app : &ChatApp) -> Option<usize>{
        for i in 0..app.save.chats.len() {
            if self.saved_id == app.save.chats[i].1{
                return Some(i);
            }
        }
        None
    }

    pub fn view<'a>(&'a self, app: &'a ChatApp) -> Element<'a, Message>{
        let index = match self.get_saved_index(app) {
            Some(x) => {x},
            None => {return text("Failed").into()},
        };
        self.view_with_index(app, index)
    }
    pub fn view_with_index<'a>(&'a self, app : &'a ChatApp, index : usize) -> Element<'a, Message>{
        keyed_column(
            app.save.chats[index].0
                .iter()
                .enumerate()
                .map(|(i, chat)| {
                    (
                        0,
                        chat.view(self, &self.markdown[i], &app.theme())
                    )
                }),
        )
        .spacing(10)
        .into()
    }
}
impl SavedChats{
    pub fn new() -> Self{
        Self(Vec::new(), generate_id(), SystemTime::now())
    }



    pub fn to_mk(&self) -> Vec<Vec<markdown::Item>>{
        return self.0.iter().map(|x| Chat::generate_mk(&x.message)).collect();
    }

    pub fn new_with_chats(chats: Vec<Chat>) -> Self{
        return Self(chats, generate_id(), SystemTime::now());
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

impl Chats{
    pub fn chat_view<'a>(&'a self, app : &'a ChatApp, id : i32) -> Element<'a, Message>{
        let input : Element<Message> = match self.state {
            State::Idle => {
                text_editor(&self.input)
                .placeholder("Type your message here...")
                .on_action(|action| Message::Chats(ChatsMessage::Action(action), self.id))
                .padding(Padding::from(20))
                .size(20)
                .style(style::text_editor::input)
                .key_binding(|key_press| {
                    let modifiers = key_press.modifiers;

                    match text_editor::Binding::from_key_press(key_press) {
                        Some(text_editor::Binding::Enter) if !modifiers.shift() => {
                            Some(text_editor::Binding::Custom(Message::Chats(ChatsMessage::Submit, self.id)))
                        }
                        binding => binding,
                    }
                })
                .into()  
            },
            State::Generating => {
                container(text("Awaiting Response...").color(app.theme().palette().primary).size(20)).padding(20).width(Length::Fill).style(container::transparent).into()
            }
            State::Listening => {
                container(text("Listening...").color(app.theme().palette().primary).size(20)).padding(20).width(Length::Fill).style(container::transparent).into()
            }
        };


        let btn = |file : &str| -> button::Button<'a, Message, Theme, Renderer>{
            button(
                svg(svg::Handle::from_path(get_path_assets(file.to_string()))).style(style::svg::primary).width(Length::Fixed(24.0)),
            )
            .style(style::button::chosen_chat)
            .width(Length::Fixed(48.0))
        };

        let upload = btn("upload.svg").on_press(Message::Chats(ChatsMessage::PickImage, self.id));
        
        let submit : Element<Message> = match self.state == State::Generating{
            true => {
                btn("close.svg").on_press(Message::StopGenerating(self.saved_id)).into()
            },
            false => {
                let record = btn("record.svg").on_press(Message::Chats(ChatsMessage::Listen, self.id));
                let send = btn("send.svg").on_press(Message::Chats(ChatsMessage::Submit, self.id));

                row![
                    record,
                    send
                ].into()
            }
        };

        //let submit = button(
        //    svg(svg::Handle::from_path(get_path_assets("send.svg".to_string()))).style(style::svg::primary).width(Length::Fixed(24.0)),
        //)
        //.style(style::button::chosen_chat)
        //
        //.width(Length::Fixed(48.0));
        
        let images = container(
            scrollable::Scrollable::new(row(self.images.iter().map(|x| {
               button(image(image::Handle::from_path(x)).height(Length::Fixed(100.0))).style(style::button::transparent_text).on_press(Message::Chats(ChatsMessage::RemoveImage(x.clone()), self.id)).into() 
            })).align_y(Vertical::Center).spacing(5)).direction(Direction::Horizontal(Scrollbar::new()))
        ).style(style::container::bottom_input_back);
        
        let bottom = container(
            row![
                upload,
                input,
                submit
            ].align_y(Vertical::Center).spacing(5),
        );

        let input = container(column![
            images,
            container(
                combo_box(
                    &app.logic.combo_models, 
                    self.model.as_str(), 
                    None,
                    move |x| Message::Chats(ChatsMessage::ChangeModel(x), id)
                )
                .input_style(style::text_input::ai_all)
                .size(12.0)
            ).width(Length::Fill).align_y(Vertical::Center).style(style::container::bottom_input_back),
            bottom, 
        ])
        .width(Length::FillPortion(10))
        .padding(Padding::from([10, 20]))
        .style(style::container::input_back);

        let input = container(input).padding(10);

        let body = match self.markdown.is_empty(){
            true => self.view_start(app),
            false => self.view_chat(app)
        };

        container(column![
            body,
            input,
        ]).width(Length::FillPortion(50)).into()
    }


    fn view_start<'a>(&'a self, app : &'a ChatApp) -> Element<'a, Message>{
        let title = text("How can I help?").size(32).color(app.theme().palette().text).align_x(Horizontal::Left);

        let colour = || -> iced::Color {
            change_alpha(lighten_colour(app.theme().palette().primary.clone(), 0.02), 0.3)
        };
        let header = row(start::SECTIONS.iter().map(|x| {
            let style = match x.title == self.start{
                true => style::button::start_chosen,
                false => style::button::start
            };

            button(
                text(x.title).color(colour()).align_x(Horizontal::Center).align_y(Vertical::Center).size(16)
            ).padding(10).style(style).on_press(Message::Chats(ChatsMessage::ChangeStart(x.title.to_string()), self.id)).into()
        }).collect::<Vec<Element<Message>>>()).spacing(10);

        let section : Vec<Section> = start::SECTIONS.into_iter().filter(|x| x.title == self.start).collect();
        let section : Section = section.first().unwrap().clone();
        
        let prompts = column(section.prompts.iter().map(|x| {
            button(
                text(x.clone()).color(colour()).align_x(Horizontal::Left).width(Length::Fill).size(16)
            ).padding(10).style(style::button::transparent_translucent).on_press(Message::Chats(ChatsMessage::Action(text_editor::Action::Edit(text_editor::Edit::Paste(Arc::new(x.to_string())))), self.id)).into()
        }).collect::<Vec<Element<Message>>>());
        
        container(row![
            horizontal_space().width(Length::FillPortion(5)),
            container(column![
                title,
                header,
                prompts
            ].spacing(20).align_x(Horizontal::Left)).width(Length::FillPortion(20)),
            horizontal_space().width(Length::FillPortion(5))
        ]).center_x(Length::Fill).center_y(Length::Fill).into()
    }

    fn view_chat<'a>(&'a self, app : &'a ChatApp) -> Element<'a, Message>{
        container(scrollable::Scrollable::new(app.save.view_chat(self,app)).width(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
    }
}
