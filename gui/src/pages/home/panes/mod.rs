use iced::{
    Element, Length, Task,
    widget::{pane_grid, text},
    window,
};

use crate::{
    Application, Message,
    pages::{Pages, home::message::HomePickingType},
};

pub mod data;
pub mod view;

#[derive(Debug, Clone)]
pub enum HomePaneType {
    Chat,
    Downloads,
    Models,
    Prompts,
    Options,
    Settings,
    Tools,
}

#[derive(Debug, Clone)]
pub enum HomePaneTypeWithId {
    Chat(u32),
    Downloads(u32),
    Models(u32),
    Prompts(u32),
    Options(u32),
    Settings(u32),
    Tools(u32),
}

impl Into<HomePaneType> for &HomePaneTypeWithId {
    fn into(self) -> HomePaneType {
        match self {
            HomePaneTypeWithId::Chat(_) => HomePaneType::Chat,
            HomePaneTypeWithId::Downloads(_) => HomePaneType::Downloads,
            HomePaneTypeWithId::Models(_) => HomePaneType::Models,
            HomePaneTypeWithId::Prompts(_) => HomePaneType::Prompts,
            HomePaneTypeWithId::Options(_) => HomePaneType::Options,
            HomePaneTypeWithId::Settings(_) => HomePaneType::Settings,
            HomePaneTypeWithId::Tools(_) => HomePaneType::Tools,
        }
    }
}

impl Into<HomePaneType> for HomePaneTypeWithId {
    fn into(self) -> HomePaneType {
        (&self).into()
    }
}

impl HomePaneType {
    pub fn new_chat(app: &mut Application) -> Self {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct HomePanes {
    pub focus: Option<pane_grid::Pane>,
    pub panes: pane_grid::State<HomePaneTypeWithId>,
    pub pick: Option<HomePickingType>,
}

impl HomePanes {
    pub fn new(pane: HomePaneTypeWithId) -> Self {
        let (panes, _) = pane_grid::State::new(pane);
        let (focus, _) = panes.panes.iter().last().unwrap();

        Self {
            focus: Some(focus.clone()),
            panes,
            pick: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PaneMessage {
    Close(pane_grid::Pane),
    PaneDragged(pane_grid::DragEvent),
    PaneResized(pane_grid::ResizeEvent),
    Split(pane_grid::Axis, pane_grid::Pane, HomePaneType),
    Replace(pane_grid::Pane, HomePaneType),
    Pick(HomePickingType),
    UnPick,
}

impl PaneMessage {
    pub fn handle(self, app: &mut Application, id: window::Id) -> Task<Message> {
        let Pages::Home(ref mut page) = app.windows.get_mut(&id).unwrap().page else {
            return Task::none();
        };

        match self {
            Self::Pick(x) => {
                page.panes.pick = Some(x);
                Task::none()
            }
            Self::UnPick => {
                page.panes.pick = None;
                Task::none()
            }
            _ => Task::none(),
        }
    }
}
