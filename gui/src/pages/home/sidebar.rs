use crate::{
    Application, InputMessage, Message,
    font::{BODY_SIZE, HEADER_SIZE, SUB_HEADING_SIZE},
    pages::{
        PageMessage,
        home::{
            HomePaneType,
            message::{HomeMessage, HomePickingType},
            panes::PaneMessage,
        },
    },
    style,
    windows::message::WindowMessage,
};
use iced::{
    Element, Length, Padding, Theme,
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container, markdown, row, text, text_input, vertical_rule, vertical_space,
    },
    window,
};
use ochat_types::{chats::previews::Preview, surreal::RecordId};

#[derive(Debug, Clone)]
pub struct PreviewMk {
    pub markdown: Vec<markdown::Item>,
    pub id: RecordId,
}

impl From<Preview> for PreviewMk {
    fn from(value: Preview) -> Self {
        Self {
            markdown: markdown::parse(&value.text).collect(),
            id: value.id,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct HomeSideBar {
    pub is_collapsed: bool,
    pub previews: Vec<PreviewMk>,
    pub search: String,
}

impl HomeSideBar {
    pub fn view<'a>(&'a self, app: &'a Application, id: window::Id) -> Element<'a, Message> {
        let mut content = row![self.pane_buttons(app, id.clone())];
        if !self.is_collapsed {
            content = content.push(self.chat_buttons(app, id));
        }

        content = content.push(vertical_rule(1));

        container(content)
            .style(style::container::side_bar)
            .width(if self.is_collapsed { 45 } else { 325 })
            .into()
    }

    fn view_preview<'a>(
        id: window::Id,
        preview: &'a PreviewMk,
        theme: &Theme,
    ) -> Element<'a, Message> {
        let title = button(
            markdown(
                preview.markdown.iter(),
                markdown::Settings::with_text_size(BODY_SIZE + 2),
                style::markdown::main(theme),
            )
            .map(|_| Message::None),
        )
        .on_press(Message::Window(WindowMessage::Page(
            id,
            PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                HomePickingType::ReplaceChat(preview.id.key().to_string()),
            ))),
        )))
        .style(style::button::transparent_back_white_text)
        .width(Length::Fill);

        let close = style::svg_button::text("close.svg", BODY_SIZE + 2).on_press(Message::Window(
            WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::DeleteChat(preview.id.key().to_string())),
            ),
        ));

        container(row![title, close].align_y(Vertical::Center)).into()
    }

    fn chat_buttons<'a>(&'a self, app: &'a Application, id: window::Id) -> Element<'a, Message> {
        let name = text("ochat")
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .style(style::text::primary)
            .size(SUB_HEADING_SIZE);

        let new_chat = button(
            text("New Chat")
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .width(Length::Fill)
                .size(HEADER_SIZE),
        )
        .on_press(Message::Window(WindowMessage::Page(
            id,
            PageMessage::Home(HomeMessage::NewChat),
        )))
        .style(style::button::rounded_primary_blend)
        .width(Length::Fill)
        .padding(Padding::from(10));

        let search = style::svg_input::primary(
            Some(String::from("search.svg")),
            text_input("Search chats...", &self.search)
                .on_input(move |x| {
                    Message::Window(WindowMessage::Page(
                        id,
                        PageMessage::Home(HomeMessage::SearchPreviews(InputMessage::Update(x))),
                    ))
                })
                .on_submit(Message::Window(WindowMessage::Page(
                    id,
                    PageMessage::Home(HomeMessage::SearchPreviews(InputMessage::Submit)),
                ))),
            SUB_HEADING_SIZE,
        );

        let previews = column(
            if self.search.is_empty() || self.previews.is_empty() {
                &app.cache.previews
            } else {
                &self.previews
            }
            .iter()
            .map(|x| Self::view_preview(id.clone(), x, &app.theme())),
        )
        .spacing(5);

        container(
            column![name, new_chat, search, previews, vertical_space()]
                .spacing(10)
                .padding(10),
        )
        .into()
    }

    fn pane_buttons<'a>(&'a self, app: &'a Application, id: window::Id) -> Element<'a, Message> {
        let size = 24;

        let collapse = style::svg_button::text(
            if self.is_collapsed {
                "panel_open.svg"
            } else {
                "panel_close.svg"
            },
            size,
        )
        .on_press(Message::Window(WindowMessage::Page(
            id,
            PageMessage::Home(HomeMessage::CollapseSideBar),
        )));

        let new_chat = style::svg_button::text("add.svg", size).on_press(Message::Window(
            WindowMessage::Page(id, PageMessage::Home(HomeMessage::NewChat)),
        ));

        let new_chat_pane = style::svg_button::text("add_chat.svg", size).on_press(
            Message::Window(WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                    HomePickingType::OpenPane(HomePaneType::Chat),
                ))),
            )),
        );

        let new_models_pane = style::svg_button::text("star.svg", size).on_press(Message::Window(
            WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                    HomePickingType::OpenPane(HomePaneType::Chat),
                ))),
            ),
        ));

        let new_prompts_pane = style::svg_button::text("prompt.svg", size).on_press(
            Message::Window(WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                    HomePickingType::OpenPane(HomePaneType::Chat),
                ))),
            )),
        );

        let new_tools_pane = style::svg_button::text("tools.svg", size).on_press(Message::Window(
            WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                    HomePickingType::OpenPane(HomePaneType::Chat),
                ))),
            ),
        ));

        let new_options_pane =
            style::svg_button::text("ai.svg", size).on_press(Message::Window(WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                    HomePickingType::OpenPane(HomePaneType::Chat),
                ))),
            )));

        let new_downloads_pane = style::svg_button::text("downloads.svg", size).on_press(
            Message::Window(WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                    HomePickingType::OpenPane(HomePaneType::Chat),
                ))),
            )),
        );

        let new_settings_pane = style::svg_button::text("settings.svg", size).on_press(
            Message::Window(WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                    HomePickingType::OpenPane(HomePaneType::Chat),
                ))),
            )),
        );

        container(
            column![
                collapse,
                new_chat,
                vertical_space(),
                new_chat_pane,
                new_models_pane,
                new_prompts_pane,
                new_tools_pane,
                new_options_pane,
                new_downloads_pane,
                new_settings_pane
            ]
            .spacing(5)
            .padding(Padding::default().top(5).bottom(5)),
        )
        .style(style::container::side_bar_darker)
        .into()
    }
}
