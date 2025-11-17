use iced::{
    Element, Length, Padding,
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container, pane_grid, row, text, text_input, vertical_rule, vertical_space,
    },
};
use ochat_types::chats::previews::Preview;

use crate::{
    Application, Message,
    font::{BODY_SIZE, HEADER_SIZE, SUB_HEADING_SIZE},
    style,
};

#[derive(Debug, Clone)]
pub struct HomePage {
    side_bar: HomeSideBar,
    panes: HomePanes,
}

impl HomePage {
    pub fn new() -> Self {
        Self {
            side_bar: HomeSideBar::default(),
            panes: HomePanes::new(HomePaneType::Chat(0)),
        }
    }

    pub fn view<'a>(&'a self, app: &'a Application) -> Element<'a, Message> {
        row![self.side_bar.view(app), self.panes.view(app)].into()
    }
}

pub enum HomeMessage {}

#[derive(Debug, Clone)]
pub enum HomePaneType {
    Chat(u32),
}

impl HomePaneType {
    pub fn new_chat(app: &mut Application) -> Self {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct HomePanes {
    pub focus: Option<pane_grid::Pane>,
    pub panes: pane_grid::State<HomePaneType>,
    pub pick: Option<HomePickingType>,
}

#[derive(Debug, Clone)]
pub enum HomePickingType {
    ReplaceChat(String),
    AddPane(HomePaneType),
}

impl HomePanes {
    pub fn new(pane: HomePaneType) -> Self {
        let (panes, _) = pane_grid::State::new(pane);
        let (focus, _) = panes.panes.iter().last().unwrap();

        Self {
            focus: Some(focus.clone()),
            panes,
            pick: None,
        }
    }

    pub fn view<'a>(&'a self, app: &'a Application) -> Element<'a, Message> {
        text("Hello, World!").width(Length::Fill).into()
    }
}

#[derive(Debug, Clone, Default)]
pub struct HomeSideBar {
    is_collapsed: bool,
    previews: Vec<Preview>,
    search: String,
}

impl HomeSideBar {
    pub fn view<'a>(&'a self, app: &'a Application) -> Element<'a, Message> {
        let mut content = row![self.pane_buttons(app)];
        if !self.is_collapsed {
            content = content.push(self.chat_buttons(app));
        }

        content = content.push(vertical_rule(1));

        container(content)
            .style(style::container::side_bar)
            .width(if self.is_collapsed { 45 } else { 325 })
            .into()
    }

    fn view_preview<'a>(preview: &'a Preview) -> Element<'a, Message> {
        let title = button(
            text(&preview.text)
                .align_x(Horizontal::Left)
                .align_y(Vertical::Center)
                .size(BODY_SIZE),
        )
        .style(style::button::transparent_back_white_text)
        .width(Length::Fill);

        let close = style::svg_button::text("close.svg", BODY_SIZE);

        container(column![title, close]).into()
    }

    fn chat_buttons<'a>(&'a self, app: &'a Application) -> Element<'a, Message> {
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
        .style(style::button::rounded_primary_blend)
        .width(Length::Fill)
        .padding(Padding::from(10));

        let search = style::svg_input::primary(
            Some(String::from("search.svg")),
            text_input("Search chats...", &self.search),
            SUB_HEADING_SIZE,
        );

        let previews = column(self.previews.iter().map(|x| Self::view_preview(x))).spacing(5);

        container(
            column![name, new_chat, search, previews, vertical_space()]
                .spacing(10)
                .padding(10),
        )
        .into()
    }

    fn pane_buttons<'a>(&'a self, app: &'a Application) -> Element<'a, Message> {
        let size = 24;

        let collapse = style::svg_button::text(
            if self.is_collapsed {
                "panel_open.svg"
            } else {
                "panel_close.svg"
            },
            size,
        );
        let new_chat = style::svg_button::text("add.svg", size);

        let new_chat_pane = style::svg_button::text("add_chat.svg", size);
        let new_models_pane = style::svg_button::text("star.svg", size);
        let new_prompts_pane = style::svg_button::text("prompt.svg", size);
        let new_tools_pane = style::svg_button::text("tools.svg", size);
        let new_options_pane = style::svg_button::text("ai.svg", size);
        let new_downloads_pane = style::svg_button::text("downloads.svg", size);
        let new_settings_pane = style::svg_button::text("settings.svg", size);

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
