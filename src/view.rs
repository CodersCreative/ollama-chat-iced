
use iced::{Subscription, Theme};
use crate::download::{pull, DownloadProgress};
use crate::download::Download;
use crate::{ChatApp, Message};
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
            id: 0,
        }
    }
    pub fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }
    //
    //pub fn subscription(&self, app : &ChatApp) -> Subscription<Message> {
    //    if let Some(download) = &self.downloading{
    //        return match download.2 {
    //            DownloadProgress::Downloading { .. } => {
    //                pull(0, download.1.clone(), app.logic.ollama.clone()).map(Message::Pulling)
    //            }
    //            _ => Subscription::none(),
    //        }
    //    }
    //    Subscription::none()
    //}
}
