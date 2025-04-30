use super::{ChatNode, Reason};
use crate::{
    chats::{chat::Role, message::ChatsMessage},
    common::Id,
    style,
    utils::get_path_assets,
    Message,
};
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container, horizontal_space, image, markdown, mouse_area, row,
        scrollable::{self, Direction, Scrollbar},
        svg, text, text_editor, Button,
    },
    Element, Length, Padding, Theme,
};

impl ChatNode {
    pub fn view_editing<'a>(
        &'a self,
        id: Id,
        content: &'a text_editor::Content,
        index: &usize,
    ) -> Element<'a, Message> {
        let images = container(
            scrollable::Scrollable::new(
                row(self.chat.images().iter().map(|x| {
                    button(image(image::Handle::from_path(x)).height(Length::Fixed(200.0)))
                        .style(style::button::transparent_text)
                        .into()
                }))
                .align_y(Vertical::Center)
                .spacing(10),
            )
            .direction(Direction::Horizontal(Scrollbar::new())),
        )
        .padding(Padding::from([0, 20]))
        .style(style::container::bottom_input_back);

        let editor = container(
            text_editor(content)
                .placeholder("Type your message here...")
                .on_action(move |action| Message::Chats(ChatsMessage::EditAction(action), id))
                .key_binding(move |key_press| {
                    let modifiers = key_press.modifiers;

                    match text_editor::Binding::from_key_press(key_press) {
                        Some(text_editor::Binding::Enter) if !modifiers.shift() => {
                            Some(text_editor::Binding::Custom(Message::Chats(
                                ChatsMessage::SaveEdit,
                                id,
                            )))
                        }
                        binding => binding,
                    }
                })
                .padding(Padding::from(20))
                .size(20)
                .style(style::text_editor::input),
        )
        .padding(20);

        container(column![self.header(&id, index), images, editor,].width(Length::Fill))
            .style(style::container::chat_back)
            .width(Length::FillPortion(5))
            .into()
    }

    fn header(&self, id: &Id, index: &usize) -> Element<Message> {
        let mut widgets = Vec::new();

        let style = match self.chat.role() == &Role::AI {
            true => style::container::chat_ai,
            false => style::container::chat,
        };

        let btn = |img: &str| -> Button<Message> {
            button(
                svg(svg::Handle::from_path(get_path_assets(img.to_string())))
                    .style(style::svg::white)
                    .width(16.0)
                    .height(16.0),
            )
            .style(style::button::transparent_text)
        };

        let txt = |txt: String| -> Element<Message> {
            text(txt)
                .size(16)
                .align_x(Horizontal::Left)
                .align_y(Vertical::Center)
                .width(Length::Fill)
                .into()
        };

        widgets.push(txt(self.chat.role().to_string()));

        if let Some(Reason::Model(x)) = &self.reason {
            widgets.push(txt(x.to_string()));
        }

        widgets.push(horizontal_space().into());

        widgets.push(
            btn("edit.svg")
                .on_press(Message::Chats(
                    ChatsMessage::Edit(self.chat.content().clone()),
                    id.clone(),
                ))
                .into(),
        );

        widgets.push(
            btn("restart.svg")
                .on_press(Message::Chats(
                    ChatsMessage::Regenerate(index.clone()),
                    id.clone(),
                ))
                .into(),
        );

        widgets.push(
            btn("copy.svg")
                .on_press(Message::SaveToClipboard(self.chat.content().to_string()))
                .into(),
        );

        if self.reason.is_some() {
            widgets.push(
                btn("back_arrow.svg")
                    .on_press(Message::Chats(
                        ChatsMessage::ChangePath(index.clone(), false),
                        id.clone(),
                    ))
                    .into(),
            );
            widgets.push(
                btn("forward_arrow.svg")
                    .on_press(Message::Chats(
                        ChatsMessage::ChangePath(index.clone(), true),
                        id.clone(),
                    ))
                    .into(),
            );
        }

        container(row(widgets).spacing(10))
            .style(style)
            .width(Length::Fill)
            .padding(3)
            .into()
    }

    pub fn view<'a>(
        &'a self,
        id: &Id,
        index: &usize,
        markdown: &'a Vec<markdown::Item>,
        theme: &Theme,
    ) -> Element<'a, Message> {
        let images = container(
            scrollable::Scrollable::new(
                row(self.chat.images().iter().map(|x| {
                    button(image(image::Handle::from_path(x)).height(Length::Fixed(200.0)))
                        .style(style::button::transparent_text)
                        .into()
                }))
                .align_y(Vertical::Center)
                .spacing(10),
            )
            .direction(Direction::Horizontal(Scrollbar::new())),
        )
        .padding(Padding::from([0, 20]))
        .style(style::container::bottom_input_back);

        let mark = mouse_area(container(self.chat.view_mk(markdown, theme)).padding(20))
            .on_right_press(Message::Chats(
                ChatsMessage::Edit(self.chat.content().clone()),
                id.clone(),
            ))
            .on_press(Message::SaveToClipboard(self.chat.content().to_string()));

        container(column![self.header(id, index), images, mark,].width(Length::Fill))
            .style(style::container::chat_back)
            .width(Length::FillPortion(5))
            .into()
    }
}
