use crate::common::Id;
use crate::download::Download;
use crate::llm::ChatStream;
use crate::models::view::Models;
use crate::options::view::Options;
use crate::prompts::view::Prompts;
use crate::settings::view::Settings;
use crate::sidebar::chats::SideChats;
use crate::sidebar::SideBarState;
use crate::tools::view::Tools;
use crate::{chats::view::Chats, llm::ChatStreamId};
use getset::{CopyGetters, Getters, MutGetters, Setters};
use iced::Theme;
use std::collections::HashMap;

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
    settings: HashMap<Id, Settings>,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    tools: HashMap<Id, Tools>,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    chats: HashMap<Id, Chats>,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    models: HashMap<Id, Models>,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    prompts: HashMap<Id, Prompts>,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    downloads: HashMap<Id, Download>,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    chat_streams: HashMap<ChatStreamId, ChatStream>,
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

    pub fn update_chat_by_saved_and_message<F>(&mut self, id: &Id, index: &usize, mut f: F)
    where
        F: FnMut(&mut Chats),
    {
        self.chats
            .iter_mut()
            .filter(|x| x.1.saved_chat() == id && x.1.chats().contains(index))
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

    pub fn add_to_settings(&mut self, key: Id, settings: Settings) {
        self.settings.insert(key, settings);
    }

    pub fn update_all_settings<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut HashMap<Id, Settings>),
    {
        f(&mut self.settings);
    }

    pub fn update_settings<F>(&mut self, key: &Id, mut f: F)
    where
        F: FnMut(Option<&mut Settings>),
    {
        f(self.settings.get_mut(key));
    }

    pub fn add_to_tools(&mut self, key: Id, tools: Tools) {
        self.tools.insert(key, tools);
    }

    pub fn update_all_tools<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut HashMap<Id, Tools>),
    {
        f(&mut self.tools);
    }

    pub fn update_tools<F>(&mut self, key: &Id, mut f: F)
    where
        F: FnMut(Option<&mut Tools>),
    {
        f(self.tools.get_mut(key));
    }

    pub fn add_model(&mut self, key: Id, models: Models) {
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

    pub fn add_prompt(&mut self, key: Id, prompts: Prompts) {
        self.prompts.insert(key, prompts);
    }

    pub fn update_prompts<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut HashMap<Id, Prompts>),
    {
        f(&mut self.prompts);
    }

    pub fn update_prompt<F>(&mut self, key: &Id, mut f: F)
    where
        F: FnMut(Option<&mut Prompts>),
    {
        f(self.prompts.get_mut(key));
    }

    pub fn add_download(&mut self, id: Id, download: Download) {
        self.downloads.insert(id, download);
    }

    pub fn update_downloads<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut HashMap<Id, Download>),
    {
        f(&mut self.downloads);
    }

    pub fn add_chat_stream(&mut self, id: ChatStreamId, stream: ChatStream) {
        self.chat_streams.insert(id, stream);
    }

    pub fn update_chat_streams<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut HashMap<ChatStreamId, ChatStream>),
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
            tools: HashMap::new(),
            settings: HashMap::new(),
            prompts: HashMap::new(),
            downloads: HashMap::new(),
            chat_streams: HashMap::new(),
        }
    }

    pub fn remove_download_by_id(&mut self, id: &Id) {
        self.update_downloads(|downloads| {
            let _ = downloads.remove(id);
        });
    }

    pub fn remove_chat_stream_by_id(&mut self, id: &ChatStreamId) {
        self.update_chat_streams(|streams| {
            let _ = streams.remove(id);
        });
    }
}
