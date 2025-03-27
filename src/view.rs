use std::{path::PathBuf, sync::Arc};

//use color_art::Color;
use iced::{
    alignment::{Horizontal, Vertical},widget::{button, column, combo_box, container, horizontal_space, row, scrollable::{self, Direction, Scrollbar}, text, text_input}, Element, Length, Padding, Renderer, Theme
};
use ollama_rs::generation::chat::ChatMessage;
use crate::{sidebar::SideBarState, start::{self, Section}, style, utils::{change_alpha, lighten_colour}, ChatApp, Message};
use crate::sidebar::chats::Chats as SideChats;
use iced::widget::image;
pub struct View{
    pub theme: Theme,
    pub loading : bool,
    pub side : SideBarState,
    pub start : String,
    pub input : String,
    pub indent : String,
    pub chats: SideChats,
    pub images: Vec<PathBuf>,
    pub gen_chats : Arc<Vec<ChatMessage>>,
}

impl View{
    pub fn new() -> Self{
        Self{
            side : SideBarState::Shown,
            start : "General".to_string(),
            theme : Theme::CatppuccinMocha,
            loading: false,
            indent: 8.to_string(),
            input: String::new(),
            chats: SideChats::new(Vec::new()),
            images: Vec::new(),
            gen_chats: Arc::new(Vec::new()),
        }
    }
    pub fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }
    
    pub fn chat_view<'a>(&'a self, app : &'a ChatApp) -> Element<'a, Message>{
        let input : Element<Message> = match self.loading {
            false => {
                text_input::<Message, Theme, Renderer>("Enter your message", &self.input)
                .on_input(Message::Edit)
                .on_submit(Message::Submit)
                .size(20)
                .padding(Padding::from(20))
                .style(style::text_input::input)
                .into()
            },
            true => {
                container(text("Awaiting Response...").color(self.theme.palette().primary).size(20)).padding(20).style(container::transparent).into()
            }
        };

        let upload = button(
            text("^").align_x(Horizontal::Center).align_y(Vertical::Center).width(Length::Fill).size(16)
        )
        .style(style::button::rounded_primary)
        .on_press(Message::PickImage)
        .width(Length::FillPortion(1));

        let submit = button(
            text(">").align_x(Horizontal::Center).align_y(Vertical::Center).width(Length::Fill).size(16)
        )
        .style(style::button::rounded_primary)
        .on_press(Message::Submit)
        .width(Length::FillPortion(2));
        let images = container(
            scrollable::Scrollable::new(row(self.images.iter().map(|x| {
               button(image(image::Handle::from_path(x)).height(Length::Fixed(100.0))).style(style::button::transparent_text).on_press(Message::RemoveImage(x.clone())).into() 
            })).align_y(Vertical::Center).spacing(10)).direction(Direction::Horizontal(Scrollbar::new()))
        ).padding(Padding::from([0, 20])).style(style::container::bottom_input_back);
        
        let bottom = container(
            row![
                container(
                    combo_box(&app.logic.models, app.save.ai_model.as_str(), None, Message::ChangeModel).input_style(style::text_input::ai_all).padding(Padding::from([5, 20]))
                ).width(Length::FillPortion(17)).align_y(Vertical::Center),//.padding(Padding::from([10, 20])),
                upload,
                submit
            ].align_y(Vertical::Center).spacing(20),
        ).padding(Padding::from([0, 20])).style(style::container::bottom_input_back);

        let input = container(column![
            images,
            input,
            bottom, 
        ])
        .width(Length::FillPortion(10))
        .padding(Padding::from(20))
        .style(style::container::input_back);

        let input = container(input).padding(Padding::default().top(5).bottom(15).left(30).right(30));

        let body = match app.markdown.is_empty(){
            true => self.view_start(app),
            false => self.view_chat(app)
        };

        container(column![
            body,
            //vertical_space(),
            input,
        ]).width(Length::FillPortion(50)).into()
    }


    fn view_start<'a>(&'a self, app : &'a ChatApp) -> Element<'a, Message>{
        let title = text("How can I help?").size(32).color(self.theme.palette().text).align_x(Horizontal::Left);

        let colour = || -> iced::Color {
            change_alpha(lighten_colour(self.theme.palette().primary.clone(), 0.02), 0.3)
        };
        let header = row(start::SECTIONS.iter().map(|x| {
            let style = match x.title == self.start{
                true => style::button::start_chosen,
                false => style::button::start
            };

            button(
                text(x.title).color(colour()).align_x(Horizontal::Center).align_y(Vertical::Center).width(Length::Fill).size(16)
            ).width(Length::FillPortion(1)).padding(10).style(style).on_press(Message::ChangeStart(x.title.to_string())).into()
        }).collect::<Vec<Element<Message>>>()).spacing(10);

        let section : Vec<Section> = start::SECTIONS.into_iter().filter(|x| x.title == self.start).collect();
        let section : Section = section.first().unwrap().clone();
        
        let prompts = column(section.prompts.iter().map(|x| {
            button(
                text(x.clone()).color(colour()).align_x(Horizontal::Left).width(Length::Fill).size(16)
            ).width(Length::Fill).padding(10).style(style::button::transparent_translucent).on_press(Message::Edit(x.to_string())).into()
        }).collect::<Vec<Element<Message>>>());
        
        container(row![
            horizontal_space(),
            column![
                title,
                header,
                prompts
            ].spacing(20).align_x(Horizontal::Left),
            horizontal_space(),
        ]).align_y(Vertical::Center).center_x(Length::Fill).center_y(Length::Fill).into()
    }

    fn view_chat<'a>(&'a self, app : &'a ChatApp) -> Element<'a, Message>{
        container(scrollable::Scrollable::new(app.save.view_chat(app)).width(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
    }
}
