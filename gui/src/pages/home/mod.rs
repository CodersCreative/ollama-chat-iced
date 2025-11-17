pub mod message;
pub mod sidebar;

use crate::{
    Application, Message,
    pages::home::{message::HomePickingType, sidebar::HomeSideBar},
};
use iced::{
    Element, Length,
    widget::{pane_grid, row, text},
    window,
};

#[derive(Debug, Clone)]
pub struct HomePage {
    pub side_bar: HomeSideBar,
    pub panes: HomePanes,
}

impl HomePage {
    pub fn new() -> Self {
        Self {
            side_bar: HomeSideBar::default(),
            panes: HomePanes::new(HomePaneType::Chat(0)),
        }
    }

    pub fn view<'a>(&'a self, app: &'a Application, id: window::Id) -> Element<'a, Message> {
        row![self.side_bar.view(app, id), self.panes.view(app)].into()
    }
}

#[derive(Debug, Clone)]
pub enum HomePaneType {
    Chat(u32),
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
