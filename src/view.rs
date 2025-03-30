use std::{path::PathBuf, sync::Arc};

use iced::{
    alignment::{Horizontal, Vertical},widget::{button, column, combo_box, container, horizontal_space, row, scrollable::{self, Direction, Scrollbar}, text, text_input}, Element, Length, Padding, Renderer, Theme
};
use ollama_rs::generation::chat::ChatMessage;
use crate::{options::Options, save::chats::Chats, sidebar::SideBarState, start::{self, Section}, style, utils::{change_alpha, lighten_colour}, ChatApp, Message};
use crate::sidebar::chats::Chats as SideChats;
use iced::widget::image;

pub struct View{
    pub theme: Theme,
    pub side : SideBarState,
    pub side_chats: SideChats,
    pub options : Vec<Options>,
    pub chats : Vec<Chats>,
}

impl View{
    pub fn new() -> Self{
        Self{
            side : SideBarState::Shown,
            theme : Theme::CatppuccinMocha,
            side_chats: SideChats::new(Vec::new()),
            options: Vec::new(),
            chats: Vec::new(),
        }
    }
    pub fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }
    
}
