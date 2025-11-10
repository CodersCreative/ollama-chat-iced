use super::{message::PromptsMessage, SavedPrompts};
use crate::{
    chats::message::ChatsMessage, common::Id, prompts::SavedPrompt, style, utils::get_path_assets,
    ChatApp, Message,
};
use derive_builder::Builder;
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container, keyed_column, markdown, row, scrollable, svg, text, text_editor,
        text_input, vertical_space, Space,
    },
    Element, Length, Padding, Renderer, Theme,
};
use std::str::FromStr;

impl SavedPrompts {
    pub fn input_view<'a>(
        &'a self,
        input: &str,
        id: &Id,
        selected: Option<usize>,
    ) -> Element<'a, Message> {
        if let Some(input) = get_command_input(input) {
            if let Ok(prompts) = self.search(input) {
                return keyed_column(prompts.iter().enumerate().map(|(i, prompt)| {
                    let theme = match selected {
                        None => style::button::transparent_text,
                        Some(x) if x == i => style::button::side_bar_chat,
                        Some(_) => style::button::transparent_text,
                    };

                    (
                        0,
                        button(text(prompt.command.clone()))
                            .on_press(Message::Chats(
                                ChatsMessage::PickedPrompt(prompt.command.clone()),
                                id.clone(),
                            ))
                            .style(theme)
                            .into(),
                    )
                }))
                .spacing(10)
                .into();
            }
        }
        Space::with_height(0).into()
    }
}

#[derive(Default, Debug, Clone, PartialEq, Builder)]
pub struct Prompt {
    pub id: Id,
    pub command: String,
    pub title: String,
    pub content: String,
}

impl From<&Edit> for Prompt {
    fn from(value: &Edit) -> Self {
        Self {
            id: value.id.clone(),
            content: value.content.text().to_string(),
            title: value.title.clone(),
            command: value.command.clone(),
        }
    }
}

impl Prompt {
    pub fn from_saved(id: Id, prompt: SavedPrompt) -> Self {
        Self {
            id,
            command: prompt.command,
            content: prompt.content,
            title: prompt.title,
        }
    }

    pub fn view<'a>(
        &'a self,
        app: &ChatApp,
        id: Id,
        expand: bool,
        edit: &'a Edit,
    ) -> Element<'a, Message> {
        let btn = |file: &str| -> button::Button<'a, Message, Theme, Renderer> {
            button(
                svg(svg::Handle::from_path(get_path_assets(file.to_string())))
                    .style(style::svg::primary)
                    .width(Length::Fixed(32.0)),
            )
            .style(style::button::chosen_chat)
            .width(Length::Fixed(48.0))
        };

        container(if !expand {
            column![
                row![
                    button(
                        text(self.title.clone())
                            .color(app.theme().palette().primary)
                            .size(24)
                            .width(Length::Fill)
                            .align_y(Vertical::Center)
                            .align_x(Horizontal::Left)
                    )
                    .style(style::button::transparent_back)
                    .padding(0)
                    .on_press(Message::Prompts(
                        PromptsMessage::Expand(self.id.clone()),
                        id.clone(),
                    )),
                    btn("delete.svg").on_press(Message::Prompts(
                        PromptsMessage::Delete(self.id.clone()),
                        id.clone(),
                    )),
                ],
                text(&self.command)
                    .color(app.theme().palette().danger)
                    .size(20)
                    .width(Length::Fill)
                    .align_y(Vertical::Center)
                    .align_x(Horizontal::Left)
            ]
        } else {
            let title = text_input::<Message, Theme, Renderer>("Enter the title", &edit.title)
                .on_input(move |x| Message::Prompts(PromptsMessage::EditTitle(x), id.clone()))
                .on_submit(Message::Prompts(PromptsMessage::EditSave, id.clone()))
                .size(16)
                .style(style::text_input::input)
                .width(Length::Fill);

            let title = container(row![
                title,
                btn("save.svg").on_press(Message::Prompts(PromptsMessage::EditSave, id.clone(),)),
                btn("close.svg").on_press(Message::Prompts(
                    PromptsMessage::Expand(self.id.clone()),
                    id.clone(),
                )),
                btn("delete.svg").on_press(Message::Prompts(
                    PromptsMessage::Delete(self.id.clone()),
                    id.clone(),
                )),
            ])
            .style(style::container::code);
            let command =
                text_input::<Message, Theme, Renderer>("Enter the command", &edit.command)
                    .on_input(move |x| Message::Prompts(PromptsMessage::EditCommand(x), id.clone()))
                    .on_submit(Message::Prompts(PromptsMessage::EditSave, id.clone()))
                    .size(16)
                    .style(style::text_input::input)
                    .width(Length::Fill);
            let content = text_editor(&edit.content)
                .placeholder("Type the prompt content here...")
                .on_action(move |action| {
                    Message::Prompts(PromptsMessage::EditAction(action), id.clone())
                })
                .padding(Padding::from(20))
                .size(20)
                .style(style::text_editor::input)
                .key_binding(move |key_press| {
                    let modifiers = key_press.modifiers;

                    match text_editor::Binding::from_key_press(key_press) {
                        Some(text_editor::Binding::Enter) if !modifiers.shift() => {
                            Some(text_editor::Binding::Custom(Message::Prompts(
                                PromptsMessage::EditSave,
                                id.clone(),
                            )))
                        }
                        binding => binding,
                    }
                });
            column![title, command, content,]
        })
        .padding(5)
        .style(style::container::side_bar)
        .into()
    }
}

pub fn get_command_input(input: &str) -> Option<&str> {
    if let Some(split) = input.split_whitespace().last() {
        if split.contains("/") {
            return Some(split.trim_start_matches("/"));
        }
    }

    None
}

#[derive(Default)]
pub struct Prompts {
    pub expand: Option<Id>,
    pub input: String,
    pub prompts: Vec<Prompt>,
    pub edit: Edit,
}

#[derive(Default)]
pub struct Edit {
    pub content: text_editor::Content,
    pub title: String,
    pub command: String,
    pub id: Id,
}

impl From<Prompt> for Edit {
    fn from(value: Prompt) -> Self {
        Self {
            content: text_editor::Content::with_text(&value.content),
            title: value.title.clone(),
            command: value.command.clone(),
            id: value.id.clone(),
        }
    }
}

impl Prompts {
    pub fn new(app: &ChatApp) -> Self {
        Self {
            expand: None,
            input: String::new(),
            prompts: app
                .prompts
                .prompts
                .iter()
                .map(|x| Prompt::from_saved(x.0.clone(), x.1.clone()))
                .collect(),
            edit: Edit::default(),
        }
    }
    pub fn view_prompts<'a>(&'a self, app: &'a ChatApp, id: Id) -> Element<'a, Message> {
        keyed_column(self.prompts.iter().enumerate().map(|(_, prompt)| {
            let mut expand = false;

            if let Some(x) = &self.expand {
                expand = x == &prompt.id;
            }

            (0, prompt.view(app, id.clone(), expand, &self.edit))
        }))
        .spacing(10)
        .into()
    }

    pub fn view<'a>(&'a self, key: Id, app: &'a ChatApp) -> Element<'a, Message> {
        let input = text_input::<Message, Theme, Renderer>("Search or Add Prompts", &self.input)
            .on_input(move |x| Message::Prompts(PromptsMessage::Input(x), key.clone()))
            .on_submit(Message::Prompts(PromptsMessage::Add, key))
            .size(16)
            .style(style::text_input::input)
            .width(Length::Fill);

        let btn = |file: &str| -> button::Button<'a, Message, Theme, Renderer> {
            button(
                svg(svg::Handle::from_path(get_path_assets(file.to_string())))
                    .style(style::svg::primary)
                    .width(Length::Fixed(32.0)),
            )
            .style(style::button::chosen_chat)
            .width(Length::Fixed(48.0))
        };

        let input = row![
            input,
            btn("upload.svg").on_press(Message::Prompts(PromptsMessage::Upload, key.clone())),
            btn("add.svg").on_press(Message::Prompts(PromptsMessage::Add, key.clone()))
        ];

        let help_text = button(text("Format your variables using brackets like this: {{variable}}. Make sure to enclose them with {{ and }}.
Utilize {{CLIPBOARD}} variable to have them replaced with clipboard content.\nPrompts can also be gotten from open-webui.")
            .color(app.theme().palette().text)
            .size(10)
            .width(Length::Fill)
            .align_y(Vertical::Center)
            .align_x(Horizontal::Left)).style(style::button::not_chosen_chat).padding(6).on_press(Message::URLClicked(markdown::Url::from_str("https://openwebui.com/prompts").unwrap()));

        container(column![
            input,
            scrollable::Scrollable::new(self.view_prompts(app, key.clone())).width(Length::Fill),
            vertical_space(),
            help_text,
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
    }
}
