// pub mod panes;
pub mod message;

use crate::{Application, Message, pages::Pages};
use iced::{Element, Size, window};

const MIN_WIDTH: f32 = 300.0;

#[derive(Debug, Clone)]
pub struct Window {
    pub page: Pages,
    pub size: Size<f32>,
}

impl Window {
    pub fn new(page: Pages) -> Self {
        Self {
            page,
            size: Size {
                width: 1920.0,
                height: 1080.0,
            },
        }
    }

    pub fn get_split_from_size(&self, size: f32) -> f32 {
        size / self.size.width
    }

    pub fn get_size_from_split(&self, split: f32) -> f32 {
        self.size.width * split
    }

    pub fn is_portrait(&self) -> bool {
        self.size.width <= MIN_WIDTH
    }

    pub fn view<'a>(&'a self, app: &'a Application, id: window::Id) -> Element<'a, Message> {
        self.page.view(app, id)
    }
}
