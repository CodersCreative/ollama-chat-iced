use iced::{
    Element, Length, Padding,
    widget::{column, container, pane_grid, row, text, vertical_rule, vertical_space},
};

use crate::{Application, Message, style};

#[derive(Debug, Clone)]
pub struct HomePage {
    side_bar: HomeSideBar,
    panes: HomePanes,
}

impl HomePage {
    pub fn new() -> Self {
        Self {
            side_bar: HomeSideBar { is_collapsed: true },
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

#[derive(Debug, Clone)]
pub struct HomeSideBar {
    is_collapsed: bool,
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

    fn chat_buttons<'a>(&'a self, app: &'a Application) -> Element<'a, Message> {
        text("Soon...").width(Length::Fill).into()
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
