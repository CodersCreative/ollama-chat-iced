
use iced::Theme;
use crate::{models::Models, options::Options, save::chats::Chats, sidebar::SideBarState};
use crate::sidebar::chats::Chats as SideChats;

pub struct View{
    pub theme: Theme,
    pub side : SideBarState,
    pub side_chats: SideChats,
    pub options : Vec<Options>,
    pub chats : Vec<Chats>,
    pub models : Vec<Models>,
    pub downloading : Option<String>,
}

impl View{
    pub fn new() -> Self{
        Self{
            side : SideBarState::Shown,
            theme : Theme::CatppuccinMocha,
            side_chats: SideChats::new(Vec::new()),
            options: Vec::new(),
            chats: Vec::new(),
            models: Vec::new(),
            downloading: None,
        }
    }
    pub fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }
    
}
