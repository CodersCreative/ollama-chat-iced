use std::time::SystemTime;

use iced::alignment::Horizontal;
use iced::alignment::Vertical;
use iced::widget::{button, column, combo_box, container, horizontal_space, image, row, scrollable, text, markdown, keyed_column, text_input, svg};
use iced::widget::scrollable::Direction;
use iced::widget::scrollable::Scrollbar;
use iced::Element;
use iced::Length;
use iced::Padding;
use iced::Task;
use iced::Theme;
use iced::widget::Renderer;
use serde::{Deserialize, Serialize};
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



#[derive(Debug, Clone,)]
pub struct Chats {
    pub markdown: Vec<Vec<markdown::Item>>,
    pub images: Vec<PathBuf>,
    pub gen_chats : Arc<Vec<ChatMessage>>,
    pub loading : bool,
    pub start : String,
    pub input : String,
    pub saved_id : i32,
    pub model : String,
    pub id : i32,
}

#[derive(Debug, Clone)]
pub enum ChatsMessage{
    Regenerate,
    Submit,
    Received(Result<ChatMessage, String>),
    ChangeModel(String),
    Edit(String),
    ChangeStart(String),
    ChangeChat(usize),
    NewChat,
    PickedImage(Result<Vec<PathBuf>, String>),
    PickImage,
    RemoveImage(PathBuf),
}

impl ChatsMessage{
    pub fn handle(&self, chats : Chats, app : &mut ChatApp) -> Task<Message>{
        match self{

            Self::Regenerate => {
                if let Some(i) = chats.get_saved_index(app){
                    app.save.chats[i].0.pop();
                    return self.submit(chats, app, false);
                }

                Task::none()
            },
            Self::RemoveImage(x) => {
                let index = Chats::get_index(app, chats.id.clone());
                if let Ok(x) = app.main_view.chats[index].images.binary_search(&x){
                    app.main_view.chats[index].images.remove(x);
                }
                Task::none()
            },
            Self::PickedImage(x) => {
                let index = Chats::get_index(app, chats.id.clone());
                if let Ok(x) = x{
                    let mut x = x.clone();
                    app.main_view.chats[index].images.append(&mut x);
                }
                Task::none()
            }
            Self::Received(x) => {
                if let Ok(x) = x{
                    return self.received(app, chats.id, x.clone());
                }
                Task::none()
            },
            Self::ChangeModel(x) => {
                let index = Chats::get_index(app, chats.id.clone());
                if !chats.loading{
                    app.main_view.chats[index].model = x.clone();
                    app.save.save(SAVE_FILE);
                    let _ = app.options.get_create_model_options_index(x.clone());
                }
                Task::none()
            },
            Self::ChangeStart(x) => {
                let index = Chats::get_index(app, chats.id.clone());
                app.main_view.chats[index].start = x.clone();
                Task::none()
            },
            Self::ChangeChat(x) => {
                if !chats.loading{
                    return self.change_chat(*x, chats, app,);
                }
                Task::none()
            },
            Self::Edit(x) => {
                let index = Chats::get_index(app, chats.id.clone());
                app.main_view.chats[index].input = x.clone();
                Task::none()
            },
            Self::NewChat => {
                if !chats.loading{
                    return Self::new_chat(app, chats.id)
                }
                Task::none()
            },
            Self::Submit => {
                self.submit(chats, app, true)
            },
            Self::PickImage => {
                ChatApp::pick_images(chats.id)
            }
        }
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
            loading: false,
            input: String::new(),
            images: Vec::new(),
            gen_chats: Arc::new(Vec::new()),
        }
    }

    pub fn get_from_id<'a>(app: &'a ChatApp, id : i32) -> &'a Self{
        app.main_view.chats.iter().find(|x| x.id == id).unwrap()
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
        let input : Element<Message> = match self.loading {
            false => {
                text_input::<Message, Theme, Renderer>("Enter your message", &self.input)
                .on_input(|x| Message::Chats(ChatsMessage::Edit(x), self.id))
                .on_submit(Message::Chats(ChatsMessage::Submit, self.id))
                .size(20)
                .padding(Padding::from(20))
                .style(style::text_input::input)
                .width(Length::Fill)
                .into()
            },
            true => {
                container(text("Awaiting Response...").color(app.theme().palette().primary).size(20)).padding(20).width(Length::Fill).style(container::transparent).into()
            }
        };

        let upload = button(
            svg(svg::Handle::from_path(get_path_assets("upload.svg".to_string()))).style(style::svg::primary).width(Length::Fixed(24.0)),
        )
        .style(style::button::chosen_chat)
        .on_press(Message::Chats(ChatsMessage::PickImage, self.id))
        .width(Length::Fixed(48.0));

        let submit = button(
            svg(svg::Handle::from_path(get_path_assets("send.svg".to_string()))).style(style::svg::primary).width(Length::Fixed(24.0)),
        )
        .style(style::button::chosen_chat)
        .on_press(Message::Chats(ChatsMessage::Submit, self.id))
        .width(Length::Fixed(48.0));
        
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
            ).padding(10).style(style::button::transparent_translucent).on_press(Message::Chats(ChatsMessage::Edit(x.to_string()), self.id)).into()
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
