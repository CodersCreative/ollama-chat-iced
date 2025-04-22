use getset::{Getters, Setters};
use iced::Theme;
use crate::common::Id;
use crate::llm::ChatStream;
use crate::chats::Chats;
use crate::download::Download;
use crate::ChatApp;
use crate::{models::Models, options::Options, sidebar::SideBarState};
use crate::sidebar::chats::Chats as SideChats;

#[derive(Getters, Setters)]
pub struct View{
    #[getset(get = "pub", set = "pub")]
    theme: Theme,
    #[getset(get = "pub", set = "pub")]
    side_state : SideBarState,
    #[getset(get = "pub", set = "pub")]
    side_chats: SideChats,
    #[getset(get = "pub", set = "pub")]
    options : Vec<Options>,
    #[getset(get = "pub", set = "pub")]
    chats : Vec<Chats>,
    #[getset(get = "pub", set = "pub")]
    models : Vec<Models>,
    #[getset(get = "pub", set = "pub")]
    downloads : Vec<Download>,
    #[getset(get = "pub", set = "pub")]
    chat_streams : Vec<ChatStream>,
    #[getset(get = "pub", set = "pub")]
    id : usize,
}

impl View{
    pub fn get_chats_mut(&mut self) -> &mut Vec<Chats> {
        &mut self.chats
    }

    pub fn add_to_chats(&mut self, chat : Chats) {
        self.chats.push(chat);
    }

    pub fn update_chats<F>(&mut self, mut f : F) where F : FnMut(&mut Vec<Chats>){        
        f(&mut self.chats);
    }
    
    pub fn update_chat<F>(&mut self, index : usize, mut f : F) where F : FnMut(&mut Chats){
        f(&mut self.chats[index]);
    }
    
    pub fn add_to_options(&mut self, options : Options){
        self.options.push(options);
    }
    
    pub fn get_options_mut(&mut self) -> &mut Vec<Options> {
        &mut self.options
    }

    pub fn update_options<F>(&mut self, mut f : F) where F : FnMut(&mut Vec<Options>){
        f(&mut self.options);
    }
    
    pub fn update_option<F>(&mut self, index : usize, mut f : F) where F : FnMut(&mut Options){
        f(&mut self.options[index]);
    }

    pub fn add_model(&mut self, models : Models){
        self.models.push(models);
    }
    pub fn get_models_mut(&mut self) -> &mut Vec<Models>{
        &mut self.models
    }
    
    pub fn update_models<F>(&mut self, mut f : F) where F : FnMut(&mut Vec<Models>){
        f(&mut self.models);
    }
    
    pub fn update_model<F>(&mut self, index : usize, mut f : F) where F : FnMut(&mut Models){
        f(&mut self.models[index]);
    }
    
    pub fn get_downloads_mut(&mut self) -> &mut Vec<Download> {
        &mut self.downloads
    } 
    
    pub fn add_download(&mut self, download : Download) {
        self.downloads.push(download);
    }
    
    pub fn update_downloads<F>(&mut self, mut f : F) where F : FnMut(&mut Vec<Download>){
        f(&mut self.downloads);
    }

    pub fn get_chat_streams_mut(&mut self) -> &mut Vec<ChatStream> {
        &mut self.chat_streams
    } 
    
    pub fn add_chat_stream(&mut self, stream : ChatStream) {
        self.chat_streams.push(stream);
    } 
    
    pub fn update_chat_streams<F>(&mut self, mut f : F) where F : FnMut(&mut Vec<ChatStream>){
        f(&mut self.chat_streams);
    }
}

impl View{
    pub fn new() -> Self{
        Self{
            side_state : SideBarState::Hidden,
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
}
