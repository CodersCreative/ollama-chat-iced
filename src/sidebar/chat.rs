use crate::common::Id;
use crate::save::chats::ChatsMessage;
use crate::{style, ChatApp, Message};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, row, text};
use iced::{Element, Length, Padding};
use std::time::SystemTime;

#[derive(Clone)]
pub struct Chat {
    title: String,
    time: SystemTime,
    id: usize,
}

impl Chat {
    pub fn new(title: String, time: SystemTime, id: usize) -> Self {
        return Self { title, time, id };
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn get_time(&self) -> &SystemTime {
        &self.time
    }

    pub fn get_id(&self) -> &usize {
        &self.id
    }

    pub fn view(&self, app: &ChatApp) -> Element<Message> {
        let style = style::button::side_bar_chat;
        let title = button(
            text(self.get_title())
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .width(Length::Fill)
                .size(20),
        )
        .style(style)
        .on_press(Message::Chats(
            ChatsMessage::ChangeChat(self.get_id().clone()),
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
        .on_press(Message::RemoveChat(self.get_id().clone()))
        .width(Length::FillPortion(1))
        .padding(Padding::from(10));
        row![title, remove,].padding(5).into()
    }
}
