use crate::chats::message::ChatsMessage;
use crate::common::Id;
use crate::{style, ChatApp, Message};
use getset::{Getters, Setters};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, row, text};
use iced::{Element, Length, Padding};
use std::time::SystemTime;

#[derive(Debug, Clone, Getters, Setters)]
pub struct SideChat {
    #[getset(get = "pub")]
    id: Id,
    #[getset(get = "pub")]
    title: String,
    #[getset(get = "pub")]
    time: SystemTime,
}

impl SideChat {
    pub fn new(id: Id, title: String, time: SystemTime) -> Self {
        return Self { id, title, time };
    }

    pub fn view(&self, app: &ChatApp) -> Element<Message> {
        let style = style::button::side_bar_chat;
        let title = button(
            text(self.title())
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .width(Length::Fill)
                .size(20),
        )
        .style(style)
        .on_press(Message::Chats(
            ChatsMessage::ChangeChat(self.id.clone()),
            app.panes.last_chat,
        ))
        .width(Length::FillPortion(7))
        .padding(Padding::from(10));

        let remove = button(
            text("x")
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .width(Length::Fill)
                .size(20),
        )
        .style(style)
        .on_press(Message::RemoveChat(self.id.clone()))
        .width(Length::FillPortion(1))
        .padding(Padding::from(10));
        row![title, remove,].padding(5).into()
    }
}
