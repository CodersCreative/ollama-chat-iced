use std::collections::HashMap;

use crate::chats::Chats;
use crate::common::Id;
use crate::download::Download;
use crate::llm::ChatStream;
use crate::sidebar::chats::Chats as SideChats;
use crate::{models::Models, options::Options, sidebar::SideBarState};
use getset::{CopyGetters, Getters, MutGetters, Setters};
use iced::Theme;

#[derive(Getters, Setters, MutGetters, CopyGetters)]
pub struct View {
    #[getset(get = "pub", set = "pub")]
    theme: Theme,
    #[getset(get = "pub", set = "pub")]
    side_state: SideBarState,
    #[getset(get = "pub", set = "pub")]
    side_chats: SideChats,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    options: HashMap<Id, Options>,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    chats: HashMap<Id, Chats>,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    models: HashMap<Id, Models>,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    downloads: Vec<Download>,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    chat_streams: Vec<ChatStream>,
    #[getset(get = "pub", set = "pub", get_copy = "pub with_prefix")]
    id: usize,
}

impl View {
    pub fn add_to_chats(&mut self, key: Id, chat: Chats) {
        self.chats.insert(key, chat);
    }

    pub fn update_chats<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut HashMap<Id, Chats>),
    {
        f(&mut self.chats);
    }

    pub fn update_chat<F>(&mut self, key: &Id, mut f: F)
    where
        F: FnMut(Option<&mut Chats>),
    {
        f(self.chats.get_mut(key));
    }

    pub fn update_chat_by_saved<F>(&mut self, id: &Id, mut f: F)
    where
        F: FnMut(&mut Chats),
    {
        self.chats
            .iter_mut()
            .filter(|x| x.1.saved_chat() == id)
            .for_each(|x| {
                f(x.1);
            });
    }

    pub fn add_to_options(&mut self, key: Id, options: Options) {
        self.options.insert(key, options);
    }

    pub fn update_options<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut HashMap<Id, Options>),
    {
        f(&mut self.options);
    }

    pub fn update_option<F>(&mut self, key: &Id, mut f: F)
    where
        F: FnMut(Option<&mut Options>),
    {
        f(self.options.get_mut(key));
    }

    pub fn add_model(&mut self, key : Id, models: Models) {
        self.models.insert(key, models);
    }

    pub fn update_models<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut HashMap<Id, Models>),
    {
        f(&mut self.models);
    }

    pub fn update_model<F>(&mut self, key: &Id, mut f: F)
    where
        F: FnMut(Option<&mut Models>),
    {
        f(self.models.get_mut(key));
    }

    pub fn add_download(&mut self, download: Download) {
        self.downloads.push(download);
    }

    pub fn update_downloads<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Vec<Download>),
    {
        f(&mut self.downloads);
    }

    pub fn add_chat_stream(&mut self, stream: ChatStream) {
        self.chat_streams.push(stream);
    }

    pub fn update_chat_streams<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Vec<ChatStream>),
    {
        f(&mut self.chat_streams);
    }
}

impl View {
    pub fn new() -> Self {
        Self {
            side_state: SideBarState::Hidden,
            theme: Theme::CatppuccinMocha,
            side_chats: SideChats::new(Vec::new()),
            options: HashMap::new(),
            chats: HashMap::new(),
            models: HashMap::new(),
            downloads: Vec::new(),
            chat_streams: Vec::new(),
            id: 0,
        }
    }

    pub fn remove_download_by_id(&mut self, id: &usize) {
        self.update_downloads(|downloads| {
            if let Some(index) = downloads.iter().position(|x| &x.id == id) {
                downloads.remove(index);
            }
        });
    }

    pub fn remove_chat_stream_by_id(&mut self, id: &Id) {
        self.update_chat_streams(|streams| {
            if let Some(index) = streams.iter().position(|x| &x.id == id) {
                streams.remove(index);
            }
        });
    }
}
