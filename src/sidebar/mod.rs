use crate::chats::message::ChatsMessage;
use crate::utils::get_path_assets;
use crate::view::View;
use crate::{style, ChatApp, Message};
use chat::SideChat;
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, checkbox, column, container, pick_list, row, scrollable, svg, text, vertical_space,
    },
    Element, Length, Padding, Renderer, Theme,
};
use std::time::{Duration, SystemTime};

pub mod chat;
pub mod chats;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SideBarState {
    Hidden,
    Shown,
    Settings,
}

impl View {
    pub fn side_bar<'a>(&'a self, app: &'a ChatApp) -> Element<'a, Message> {
        match app.main_view.side_state() {
            SideBarState::Shown => self.full_side_bar(app),
            SideBarState::Hidden => self.hidden_side_bar(app),
            SideBarState::Settings => self.settings_side_bar(app),
        }
    }

    pub fn hidden_side_bar<'a>(&'a self, app: &'a ChatApp) -> Element<'a, Message> {
        let show = Self::hide_button("panel_open.svg").width(Length::Fill);

        let settings = Self::settings_button().width(Length::Fill);

        let new = Self::add_button(app).width(Length::Fill);

        container(column![show, vertical_space(), new, settings,])
            .width(Length::Fixed(48.0))
            .height(Length::Fill)
            .align_y(iced::alignment::Vertical::Bottom)
            .align_x(Horizontal::Center)
            .style(style::container::side_bar)
            .into()
    }

    fn get_length(potrait: bool) -> Length {
        if potrait {
            Length::Fill
        } else {
            Length::Fixed(300.0)
        }
    }

    pub fn settings_side_bar<'a>(&'a self, app: &'a ChatApp) -> Element<'a, Message> {
        container(column![
            self.header("Settings".to_string()),
            container(
                pick_list(Theme::ALL, Some(self.theme()), Message::ChangeTheme,)
                    .width(Length::Fill)
            )
            .padding(10),
            Self::txt("Downloads".to_string(), app.theme().palette().primary),
            self.get_downloads(app),
            vertical_space(),
            checkbox("Use Panes", app.save.use_panes).on_toggle(Message::ChangeUsePanels),
        ])
        .width(Self::get_length(app.potrait))
        .style(style::container::side_bar)
        .into()
    }

    fn get_downloads<'a>(&'a self, app: &'a ChatApp) -> Element<'a, Message> {
        if app.main_view.downloads().is_empty() {
            return Self::txt("None".to_string(), self.theme().palette().text);
        }

        column(
            app.main_view
                .downloads()
                .iter()
                .map(|(i, x)| x.view(app, i.clone())),
        )
        .into()
    }

    fn hide_button<'a>(title: &'a str) -> button::Button<'a, Message, Theme, Renderer> {
        button(
            svg(svg::Handle::from_path(get_path_assets(title.to_string())))
                .style(style::svg::white)
                .width(24.0)
                .height(24.0),
        )
        .style(style::button::transparent_text)
        .on_press(Message::SideBar)
    }

    fn settings_button<'a>() -> button::Button<'a, Message, Theme, Renderer> {
        button(
            svg(svg::Handle::from_path(get_path_assets(
                "settings.svg".to_string(),
            )))
            .style(style::svg::white)
            .width(24.0)
            .height(24.0),
        )
        .style(style::button::transparent_text)
        .on_press(Message::ShowSettings)
    }

    fn add_button<'a>(app: &'a ChatApp) -> button::Button<'a, Message, Theme, Renderer> {
        button(
            svg(svg::Handle::from_path(get_path_assets(
                "add_chat.svg".to_string(),
            )))
            .style(style::svg::white)
            .width(24.0)
            .height(24.0),
        )
        .style(style::button::transparent_text)
        .on_press(Message::Chats(ChatsMessage::NewChat, app.panes.last_chat))
    }

    fn header<'a>(&'a self, title: String) -> Element<'a, Message> {
        let palette = self.theme().palette();

        let hide = Self::hide_button("panel_close.svg").width(Length::FillPortion(2));

        let settings = Self::settings_button().width(Length::FillPortion(2));

        return container(
            row![
                hide,
                text(title)
                    .color(palette.primary)
                    .size(24)
                    .width(Length::FillPortion(6))
                    .align_y(Vertical::Center)
                    .align_x(Horizontal::Center),
                settings,
            ]
            .align_y(Vertical::Center),
        )
        .width(Length::Fill)
        .center_x(Length::Fill)
        .into();
    }

    pub fn full_side_bar<'a>(&'a self, app: &'a ChatApp) -> Element<'a, Message> {
        let new_button = button(
            text("New Chat")
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .width(Length::Fill)
                .size(24),
        )
        .style(style::button::rounded_primary)
        .on_press(Message::Chats(ChatsMessage::NewChat, app.panes.last_chat))
        .width(Length::Fill)
        .padding(Padding::from(10));

        let new_button = container(new_button)
            .padding(Padding::from(10))
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center);
        container(column![
            self.header("ochat".to_string()),
            new_button,
            self.view_chats(app),
            vertical_space(),
        ])
        .width(Self::get_length(app.potrait))
        .style(style::container::side_bar)
        .into()
    }

    fn txt<'a>(title: String, color: iced::Color) -> Element<'a, Message> {
        text(title)
            .color(color)
            .size(16)
            .width(Length::FillPortion(6))
            .align_y(Vertical::Center)
            .align_x(Horizontal::Center)
            .into()
    }

    pub fn view_chats<'a>(&'a self, app: &'a ChatApp) -> Element<'a, Message> {
        if app.main_view.side_chats().chats.len() >= 100 {
            let view = |chats: Vec<&'a SideChat>| -> Element<Message> {
                let chats: Vec<Element<Message>> =
                    chats.iter().map(|x| x.view(app)).clone().collect();
                return scrollable(column(chats).spacing(2)).into();
            };

            return column![
                Self::txt("This Month".to_string(), self.theme().palette().primary),
                view(
                    (&app.main_view.side_chats().chats)
                        .iter()
                        .filter(|x| x
                            .time()
                            .duration_since(SystemTime::now())
                            .unwrap_or(Duration::new(0, 0))
                            .as_secs()
                            < 2629746)
                        .collect::<Vec<&SideChat>>()
                ),
                Self::txt("Old".to_string(), self.theme().palette().primary),
                view(
                    (&app.main_view.side_chats().chats)
                        .iter()
                        .filter(|x| x
                            .time()
                            .duration_since(SystemTime::now())
                            .unwrap_or(Duration::new(0, 0))
                            .as_secs()
                            > 2629746)
                        .collect::<Vec<&SideChat>>()
                ),
            ]
            .into();
        } else {
            return column![
                Self::txt("All".to_string(), self.theme().palette().primary),
                container(app.main_view.side_chats().view(app)),
            ]
            .into();
        }
    }
}
