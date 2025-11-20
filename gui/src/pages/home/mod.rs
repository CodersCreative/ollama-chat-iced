pub mod message;
pub mod panes;
pub mod sidebar;

use crate::{
    Application, Message,
    pages::home::{
        panes::{HomePaneType, HomePanes},
        sidebar::HomeSideBar,
    },
};
use iced::{Element, widget::row, window};

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
