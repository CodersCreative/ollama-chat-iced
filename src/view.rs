use iced::Theme;
use crate::chat::ChatStream;
use crate::download::Download;
use crate::{models::Models, options::Options, save::chats::Chats, sidebar::SideBarState};
use crate::sidebar::chats::Chats as SideChats;

pub struct View{
    pub theme: Theme,
    pub side : SideBarState,
    pub side_chats: SideChats,
    pub options : Vec<Options>,
    pub chats : Vec<Chats>,
    pub models : Vec<Models>,
    pub downloads : Vec<Download>,
    pub chat_streams : Vec<ChatStream>,
    pub id : usize,
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
            downloads: Vec::new(),
            chat_streams: Vec::new(),
            id: 0,
        }
    }
    pub fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }
}
