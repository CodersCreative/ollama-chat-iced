use crate::{
    Application, InputMessage, Message,
    font::{HEADER_SIZE, SUB_HEADING_SIZE, get_bold_font},
    pages::{
        PageMessage,
        home::{
            COLLAPSED_CUT_OFF, HomePaneType, NORMAL_SIZE,
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
        Button, button, column, container, hover, markdown, right, row, rule, space, text,
        text_input,
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

#[derive(Debug, Clone)]
pub struct HomeSideBar {
    pub split: f32,
    pub is_collapsed: bool,
    pub previews: Vec<PreviewMk>,
    pub search: String,
}

impl Default for HomeSideBar {
    fn default() -> Self {
        Self {
            split: NORMAL_SIZE,
            is_collapsed: false,
            previews: Vec::new(),
            search: String::new(),
        }
    }
}

impl HomeSideBar {
    pub fn view<'a>(&'a self, app: &'a Application, id: window::Id) -> Element<'a, Message> {
        let mut content = row![self.pane_buttons(app, id.clone())];
        if self.split > COLLAPSED_CUT_OFF || !self.is_collapsed {
            content = content.push(self.chat_buttons(app, id));
        }

        content = content.push(rule::vertical(2).style(style::rule::side_bar_darker));

        container(content).style(style::container::side_bar).into()
    }

    fn view_preview<'a>(
        id: window::Id,
        preview: &'a PreviewMk,
        theme: &Theme,
    ) -> Element<'a, Message> {
        let title = button(markdown::view_with(
            preview.markdown.iter(),
            style::markdown::main(theme),
            &style::markdown::CustomViewer,
        ))
        .clip(true)
        .on_press(Message::Window(WindowMessage::Page(
            id,
            PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                HomePickingType::ReplaceChat(preview.id.key().to_string()),
            ))),
        )))
        .style(style::button::transparent_back_white_text)
        .width(Length::Fill);

        let close = style::svg_button::text("close.svg", SUB_HEADING_SIZE)
            .height(Length::Fill)
            .on_press(Message::Window(WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::DeleteChat(preview.id.key().to_string())),
            )));

        container(hover(title, right(close).align_y(Vertical::Center))).into()
    }

    fn chat_buttons<'a>(&'a self, app: &'a Application, id: window::Id) -> Element<'a, Message> {
        let name = text("ochat")
            .font(get_bold_font())
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .style(style::text::primary)
            .size(SUB_HEADING_SIZE);

        let new_chat = button(
            text("New Chat")
                .font(get_bold_font())
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
            column![name, new_chat, search, previews, space::vertical()]
                .spacing(10)
                .padding(10),
        )
        .into()
    }

    fn pane_buttons_vec<'a>(&'a self, id: window::Id, size: u32) -> Vec<Button<'a, Message>> {
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
                    HomePickingType::OpenPane(HomePaneType::Models),
                ))),
            ),
        ));

        let new_prompts_pane = style::svg_button::text("prompt.svg", size).on_press(
            Message::Window(WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                    HomePickingType::OpenPane(HomePaneType::Prompts),
                ))),
            )),
        );

        let new_tools_pane = style::svg_button::text("tools.svg", size).on_press(Message::Window(
            WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                    HomePickingType::OpenPane(HomePaneType::Tools),
                ))),
            ),
        ));

        let new_options_pane =
            style::svg_button::text("ai.svg", size).on_press(Message::Window(WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                    HomePickingType::OpenPane(HomePaneType::Options),
                ))),
            )));

        let new_pulls_pane = style::svg_button::text("downloads.svg", size).on_press(
            Message::Window(WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                    HomePickingType::OpenPane(HomePaneType::Pulls),
                ))),
            )),
        );

        let new_settings_pane = style::svg_button::text("settings.svg", size).on_press(
            Message::Window(WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                    HomePickingType::OpenPane(HomePaneType::Settings),
                ))),
            )),
        );

        let quit = style::svg_button::danger("quit.svg", size).on_press(Message::Quit);

        vec![
            new_chat_pane,
            new_models_pane,
            new_prompts_pane,
            new_tools_pane,
            new_options_pane,
            new_pulls_pane,
            new_settings_pane,
            quit,
        ]
    }

    fn pane_buttons<'a>(&'a self, _app: &'a Application, id: window::Id) -> Element<'a, Message> {
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

        let mut col = column![collapse, new_chat, space::vertical()]
            .spacing(5)
            .padding(Padding::default().top(5).bottom(5));

        for button in self.pane_buttons_vec(id, size) {
            col = col.push(button);
        }

        container(col)
            .style(style::container::side_bar_darker)
            .into()
    }
}
