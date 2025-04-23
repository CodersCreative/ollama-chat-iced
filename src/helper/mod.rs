pub mod chats;
pub mod image;

use crate::{sidebar::SideBarState, ChatApp, Message, SAVE_FILE};
use iced::{Task, Theme};

impl ChatApp {
    pub fn change_theme(&mut self, theme: Theme) -> Task<Message> {
        self.main_view.set_theme(theme.clone());
        let mut index = None;

        if let Some(i) = Theme::ALL.iter().position(|x| x == &theme) {
            index = Some(i);
        }

        self.save.theme = index;
        self.save.save(SAVE_FILE);
        Task::none()
    }

    pub fn toggle_side_bar_state(&mut self, target_state: SideBarState) {
        self.main_view
            .set_side_state(match self.main_view.side_state() {
                s if s == &target_state => SideBarState::Shown,
                _ => target_state,
            });
    }
}
