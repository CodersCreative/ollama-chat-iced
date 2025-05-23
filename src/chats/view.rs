use super::message::ChatsMessage;
use super::TooledOptions;
use crate::common::Id;
use crate::prompts::view::get_command_input;
use crate::start::{self, Section};
use crate::style;
use crate::utils::{change_alpha, get_path_assets, lighten_colour};
use crate::{ChatApp, Message};
use getset::{CopyGetters, Getters, MutGetters, Setters};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::mouse_area;
use iced::widget::text_editor::Motion;
use iced::widget::{
    button, column, combo_box, container, horizontal_space, image, keyed_column, markdown, row,
    scrollable,
    scrollable::{Direction, Scrollbar},
    svg, text, text_editor, Renderer,
};
use iced::{Element, Length, Padding, Theme};
use std::{path::PathBuf, sync::Arc};

#[derive(Debug, Getters, Setters, MutGetters, CopyGetters)]
pub struct Chats {
    #[getset(get = "pub", set = "pub")]
    markdown: Vec<Vec<markdown::Item>>,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    edit: text_editor::Content,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    images: Vec<PathBuf>,
    #[getset(get = "pub", set = "pub")]
    state: State,
    #[getset(get = "pub", set = "pub")]
    start: String,
    #[getset(get = "pub", set = "pub")]
    content: text_editor::Content,
    #[getset(get = "pub", set = "pub")]
    saved_chat: Id,
    #[getset(get = "pub", set = "pub")]
    selected_prompt: Option<usize>,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    models: Vec<String>,
    #[getset(get = "pub", set = "pub")]
    desc: Option<String>,
    #[getset(get = "pub", set = "pub")]
    tools: Arc<TooledOptions>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    Generating,
    Listening,
    Idle,
}

impl Clone for Chats {
    fn clone(&self) -> Self {
        Self::new(
            self.models.clone(),
            self.saved_chat().clone(),
            self.markdown.clone(),
        )
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone();
    }
}

impl Chats {
    pub fn content_perform(&mut self, action: text_editor::Action) {
        self.content.perform(action);
    }

    pub fn add_markdown(&mut self, markdown: Vec<markdown::Item>) {
        self.markdown.push(markdown);
    }

    pub fn update_markdown<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Vec<Vec<markdown::Item>>),
    {
        f(&mut self.markdown);
    }

    pub fn set_content_text(&mut self, content: &str) {
        self.content = text_editor::Content::with_text(content);
    }

    pub fn get_content_text(&self) -> String {
        self.content.text()
    }

    pub fn add_image(&mut self, image: PathBuf) {
        self.images.push(image);
    }

    pub fn update_images<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Vec<PathBuf>),
    {
        f(&mut self.images);
    }

    pub fn add_images(&mut self, images: &mut Vec<PathBuf>) {
        self.images.append(images);
    }
}

impl Chats {
    pub fn new(models: Vec<String>, saved_chat: Id, markdown: Vec<Vec<markdown::Item>>) -> Self {
        Self {
            models,
            saved_chat,
            markdown,
            edit: text_editor::Content::new(),
            start: "General".to_string(),
            state: State::Idle,
            content: text_editor::Content::new(),
            images: Vec::new(),
            desc: None,
            selected_prompt: None,
            tools: Arc::new(TooledOptions::default()),
        }
    }

    fn check_is_edit(app: &ChatApp, index: &usize, id: &Id) -> bool {
        let edit = app.main_view.edits().get(id);

        if let Some(edit) = edit {
            return index == edit;
        }

        false
    }

    pub fn view<'a>(&'a self, app: &'a ChatApp, id: &Id) -> Element<'a, Message> {
        if let Some(chat) = app.chats.0.get(self.saved_chat()) {
            return keyed_column(chat.chats.into_iter().enumerate().map(|(i, chat)| {
                (
                    0,
                    match Self::check_is_edit(app, &i, id) {
                        false => {
                            if let Some(mk) = self.markdown.get(i) {
                                chat.view(id, &i, mk, &app.theme())
                            } else {
                                text("Failed!").into()
                            }
                        }
                        true => chat.view_editing(id.clone(), &self.edit, &i),
                    },
                )
            }))
            .spacing(10)
            .into();
        }
        text("Failed to get Chat.").into()
    }

    pub fn chat_view<'a>(&'a self, app: &'a ChatApp, id: Id) -> Element<'a, Message> {
        let input: Element<Message> = match self.state {
            State::Idle => text_editor(self.content())
                .placeholder("Type your message here...")
                .on_action(move |action| Message::Chats(ChatsMessage::Action(action), id))
                .padding(Padding::from(20))
                .size(20)
                .style(style::text_editor::input)
                .key_binding(move |key_press| {
                    let modifiers = key_press.modifiers;

                    let is_command = get_command_input(&self.content().text()).is_some();

                    match text_editor::Binding::from_key_press(key_press) {
                        Some(text_editor::Binding::Enter) if !modifiers.shift() && is_command => {
                            Some(text_editor::Binding::Custom(Message::Chats(
                                ChatsMessage::SubmitPrompt,
                                id,
                            )))
                        }
                        Some(text_editor::Binding::Move(Motion::Up))
                            if !modifiers.shift() && is_command =>
                        {
                            Some(text_editor::Binding::Custom(Message::Chats(
                                ChatsMessage::ChangePrompt(Motion::Up),
                                id,
                            )))
                        }
                        Some(text_editor::Binding::Move(Motion::Down))
                            if !modifiers.shift() && is_command =>
                        {
                            Some(text_editor::Binding::Custom(Message::Chats(
                                ChatsMessage::ChangePrompt(Motion::Down),
                                id,
                            )))
                        }
                        Some(text_editor::Binding::Enter) if !modifiers.shift() => Some(
                            text_editor::Binding::Custom(Message::Chats(ChatsMessage::Submit, id)),
                        ),
                        binding => binding,
                    }
                })
                .into(),
            State::Generating => container(
                text("Awaiting Response...")
                    .color(app.theme().palette().primary)
                    .size(20),
            )
            .padding(20)
            .width(Length::Fill)
            .style(container::transparent)
            .into(),
            State::Listening => container(
                text("Listening...")
                    .color(app.theme().palette().primary)
                    .size(20),
            )
            .padding(20)
            .width(Length::Fill)
            .style(container::transparent)
            .into(),
        };

        let btn = |file: &str| -> button::Button<'a, Message, Theme, Renderer> {
            button(
                svg(svg::Handle::from_path(get_path_assets(file.to_string())))
                    .style(style::svg::primary)
                    .width(Length::Fixed(24.0)),
            )
            .style(style::button::chosen_chat)
            .width(Length::Fixed(48.0))
        };

        let btn_small = |file: &str| -> button::Button<'a, Message, Theme, Renderer> {
            button(
                svg(svg::Handle::from_path(get_path_assets(file.to_string())))
                    .style(style::svg::primary)
                    .width(Length::Fixed(12.0)),
            )
            .style(style::button::chosen_chat)
            .width(Length::Fixed(36.0))
        };

        let upload = btn("upload.svg").on_press(Message::Chats(ChatsMessage::PickImage, id));

        let submit: Element<Message> = match self.state == State::Generating {
            true => btn("close.svg")
                .on_press(Message::StopGenerating(self.saved_chat().clone()))
                .into(),
            false => {
                let send = btn("send.svg")
                    .on_press(Message::Chats(ChatsMessage::Submit, id))
                    .into();

                let mut widgets: Vec<Element<Message>> = vec![send];

                #[cfg(feature = "voice")]
                {
                    let call = btn("call.svg")
                        .on_press(Message::Call(crate::call::CallMessage::StartCall(
                            self.models[0].clone(),
                        )))
                        .into();
                    let record = btn("record.svg")
                        .on_press(Message::Chats(ChatsMessage::Listen, id))
                        .into();
                    widgets.push(call);
                    widgets.push(record);
                }

                widgets.reverse();
                row(widgets).into()
            }
        };

        let images = container(
            scrollable::Scrollable::new(
                row(self.images.iter().map(|x| {
                    button(image(image::Handle::from_path(x)).height(Length::Fixed(100.0)))
                        .style(style::button::transparent_text)
                        .on_press(Message::Chats(ChatsMessage::RemoveImage(x.clone()), id))
                        .into()
                }))
                .align_y(Vertical::Center)
                .spacing(5),
            )
            .direction(Direction::Horizontal(Scrollbar::new())),
        )
        .style(style::container::bottom_input_back);

        let bottom = container(
            row![upload, input, submit]
                .align_y(Vertical::Center)
                .spacing(5),
        )
        .max_height(350);

        let input = container(column![
            images,
            self.view_commands(app, &id),
            container(row![
                scrollable::Scrollable::new(row(self.models().iter().enumerate().map(
                    |(i, model)| {
                        mouse_area(
                            combo_box(&app.logic.combo_models, model, None, move |x| {
                                Message::Chats(ChatsMessage::ChangeModel(i, x), id)
                            })
                            .input_style(style::text_input::ai_all)
                            .size(12.0),
                        )
                        .on_right_press(Message::Chats(ChatsMessage::RemoveModel(i), id))
                        .into()
                    }
                )))
                .width(Length::Fill),
                btn_small("add.svg").on_press(Message::Chats(ChatsMessage::AddModel, id)),
            ])
            .width(Length::Fill)
            .align_y(Vertical::Center)
            .style(style::container::bottom_input_back),
            bottom,
        ])
        .width(Length::FillPortion(10))
        .padding(Padding::from([10, 20]))
        .style(style::container::input_back);

        let input = container(input).padding(10);

        let body = match self.markdown.is_empty() {
            true => self.view_start(app, id.clone()),
            false => self.view_chat(app, &id),
        };

        container(column![body, input,])
            .width(Length::FillPortion(50))
            .into()
    }

    fn view_start<'a>(&'a self, app: &'a ChatApp, id: Id) -> Element<'a, Message> {
        let title = text("How can I help?")
            .size(32)
            .color(app.theme().palette().text)
            .align_x(Horizontal::Left);

        let colour = || -> iced::Color {
            change_alpha(
                lighten_colour(app.theme().palette().primary.clone(), 0.02),
                0.3,
            )
        };
        let header = row(start::SECTIONS
            .iter()
            .map(|x| {
                let style = match x.title == self.start {
                    true => style::button::start_chosen,
                    false => style::button::start,
                };

                button(
                    text(x.title)
                        .color(colour())
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
                        .size(16),
                )
                .padding(10)
                .style(style)
                .on_press(Message::Chats(
                    ChatsMessage::ChangeStart(x.title.to_string()),
                    id,
                ))
                .into()
            })
            .collect::<Vec<Element<Message>>>())
        .spacing(10);

        let section: Vec<Section> = start::SECTIONS
            .into_iter()
            .filter(|x| x.title == self.start)
            .collect();
        let section: Section = section.first().unwrap().clone();

        let prompts = column(
            section
                .prompts
                .iter()
                .map(|x| {
                    button(
                        text(x.clone())
                            .color(colour())
                            .align_x(Horizontal::Left)
                            .width(Length::Fill)
                            .size(16),
                    )
                    .padding(10)
                    .style(style::button::transparent_translucent)
                    .on_press(Message::Chats(
                        ChatsMessage::Action(text_editor::Action::Edit(text_editor::Edit::Paste(
                            Arc::new(x.to_string()),
                        ))),
                        id,
                    ))
                    .into()
                })
                .collect::<Vec<Element<Message>>>(),
        );

        container(row![
            horizontal_space().width(Length::FillPortion(5)),
            container(
                column![title, header, prompts]
                    .spacing(20)
                    .align_x(Horizontal::Left)
            )
            .width(Length::FillPortion(20)),
            horizontal_space().width(Length::FillPortion(5))
        ])
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }

    fn view_commands<'a>(&'a self, app: &'a ChatApp, id: &Id) -> Element<'a, Message> {
        container(
            scrollable::Scrollable::new(app.prompts.input_view(
                &app.main_view.chats().get(id).unwrap().content.text(),
                id,
                self.selected_prompt,
            ))
            .width(Length::Fill),
        )
        .max_height(250)
        .into()
    }

    fn view_chat<'a>(&'a self, app: &'a ChatApp, id: &Id) -> Element<'a, Message> {
        container(
            scrollable::Scrollable::new(self.view(app, id))
                .width(Length::Fill)
                .anchor_bottom(),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
    }
}
