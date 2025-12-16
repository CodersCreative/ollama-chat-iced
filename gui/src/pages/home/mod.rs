pub mod message;
pub mod panes;
pub mod sidebar;

use crate::{
    Application, Message,
    pages::{
        PageMessage,
        home::{
            panes::{HomePaneType, HomePaneTypeWithId, HomePanes},
            sidebar::HomeSideBar,
        },
    },
    windows::message::WindowMessage,
};
use iced::{Element, window};
use iced_split::vertical_split;

pub const COLLAPSED_SIZE: f32 = 50.0;
pub const COLLAPSED_CUT_OFF: f32 = 100.0;
pub const NORMAL_SIZE: f32 = 350.0;

#[derive(Debug, Clone)]
pub struct HomePage {
    pub side_bar: HomeSideBar,
    pub panes: HomePanes,
}

impl HomePage {
    pub fn new() -> Self {
        Self {
            side_bar: HomeSideBar::default(),
            panes: HomePanes::new(HomePaneTypeWithId::Loading),
        }
    }

    pub fn view<'a>(&'a self, app: &'a Application, id: window::Id) -> Element<'a, Message> {
        vertical_split(
            self.side_bar.view(app, id),
            self.panes.view(app, id),
            app.windows.get(&id).unwrap().get_split_from_size(
                if self.side_bar.is_collapsed || self.side_bar.split <= COLLAPSED_SIZE {
                    COLLAPSED_SIZE
                } else if (NORMAL_SIZE - 25.0) <= self.side_bar.split
                    && (NORMAL_SIZE + 25.0) >= self.side_bar.split
                {
                    NORMAL_SIZE
                } else {
                    self.side_bar.split
                },
            ),
            move |x| {
                Message::Window(WindowMessage::Page(
                    id,
                    PageMessage::Home(message::HomeMessage::SplitDrag(x)),
                ))
            },
        )
        .into()
    }
}
