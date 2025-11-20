use iced::{
    Element, Length,
    widget::{pane_grid, text},
};

use crate::{Application, Message, pages::home::message::HomePickingType};

pub mod data;
pub mod view;

#[derive(Debug, Clone)]
pub enum HomePaneType {
    Chat(u32),
    Downloads(u32),
    Models(u32),
    Prompts(u32),
    Options(u32),
    Settings(u32),
    Tools(u32),
}

impl HomePaneType {
    pub fn new_chat(app: &mut Application) -> Self {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct HomePanes {
    pub focus: Option<pane_grid::Pane>,
    pub panes: pane_grid::State<HomePaneType>,
    pub pick: Option<HomePickingType>,
}

impl HomePanes {
    pub fn new(pane: HomePaneType) -> Self {
        let (panes, _) = pane_grid::State::new(pane);
        let (focus, _) = panes.panes.iter().last().unwrap();

        Self {
            focus: Some(focus.clone()),
            panes,
            pick: None,
        }
    }

    pub fn view<'a>(&'a self, app: &'a Application) -> Element<'a, Message> {
        text("Hello, World!").width(Length::Fill).into()
    }
}
