pub mod chat;
pub mod chats;

use iced::{Theme, Command};
use crate::{ChatApp, Message, SAVE_FILE};

impl ChatApp{
    pub fn change_theme(&mut self, theme : Theme) -> Command<Message>{
        self.main_view.theme = theme.clone();
        let mut index = None;

        for i in 0..Theme::ALL.len(){
            if Theme::ALL[i] == theme{
                index = Some(i);
                break;
            }
        }

        self.save.theme = index;
        self.save.save(SAVE_FILE);
        Command::none()
    }
}
