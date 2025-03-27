use std::time::{Duration, SystemTime};

use crate::sidebar::chat::Chat;
use iced::{
    alignment::{Horizontal, Vertical},widget::{button, column, combo_box, container,pick_list, row, scrollable, text, vertical_space}, Element, Length, Padding, Renderer, Theme
};
use crate::{style, ChatApp, Message};

use crate::view::View;

pub mod chats;
pub mod chat;

pub enum SideBarState {
    Hidden,
    Shown,
    Settings,
}

impl View{
    pub fn side_bar<'a>(&'a self, app : &'a ChatApp) -> Element<Message>{
        match app.main_view.side{
            SideBarState::Shown => self.full_side_bar(app),
            SideBarState::Hidden => self.hidden_side_bar(app),
            SideBarState::Settings => self.settings_side_bar(app),
        }
    }
    
    pub fn hidden_side_bar<'a>(&'a self, app : &'a ChatApp) -> Element<Message>{
        let show = Self::hide_button(">")
        .width(Length::FillPortion(2));
        
        let settings = Self::settings_button()
        .width(Length::FillPortion(2));
        
        let new = Self::add_button()
        .width(Length::FillPortion(2));
        
        container(column![
            show,
            vertical_space(),
            new,
            settings,
        ]).width(Length::FillPortion(1)).height(Length::Fill).align_y(iced::alignment::Vertical::Bottom)
        .style(style::container::side_bar).into()
    }

    pub fn settings_side_bar<'a>(&'a self, app : &'a ChatApp) -> Element<Message>{
        container(column![
            self.header("Settings".to_string()),
            text("Model Options")
            .color(self.theme().palette().primary)
            .size(16)
            .width(Length::FillPortion(6))
            .align_y(Vertical::Center)
            .align_x(Horizontal::Center),
            container(combo_box(&app.logic.models, app.save.ai_model.as_str(), None, Message::ChangeModel)).padding(10),
            container(app.options.view()),
            vertical_space(),
            container(
                pick_list(
                    Theme::ALL,
                    Some(self.theme()),
                    Message::ChangeTheme,
                ).width(Length::Fill)
            ).padding(10),
        ]).width(Length::FillPortion(10))
        .style(style::container::side_bar).into()
    }

    fn hide_button<'a>(title: &'a str) -> button::Button<'a, Message, Theme, Renderer>{
        button(
            text(title).align_x(Horizontal::Center).align_y(Vertical::Center).width(Length::Fill).size(24)
        )
        .style(style::button::transparent_text)
        .on_press(Message::SideBar)
    }

    fn settings_button<'a>() -> button::Button<'a, Message, Theme, Renderer>{
        button(
            text("=").align_x(Horizontal::Center).align_y(Vertical::Center).width(Length::Fill).size(24)
        )
        .style(style::button::transparent_text)
        .on_press(Message::ShowSettings)
    }
    
    fn add_button<'a>() -> button::Button<'a, Message, Theme, Renderer>{
        button(
            text("+").align_x(Horizontal::Center).align_y(Vertical::Center).width(Length::Fill).size(24)
        )
        .style(style::button::transparent_text)
        .on_press(Message::NewChat)
    }

    fn header<'a>(&'a self, title: String) -> Element<Message>{
        let palette = self.theme().palette();
        
        let hide = Self::hide_button("<")
        .width(Length::FillPortion(2));
        
        let settings = Self::settings_button()
        .width(Length::FillPortion(2));

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
            ].align_y(Vertical::Center)
        ).width(Length::Fill).center_x(Length::Fill).into();

    }

    pub fn full_side_bar<'a>(&'a self, app : &'a ChatApp) -> Element<Message>{
        let new_button = button(
            text("New Chat").align_x(Horizontal::Center).align_y(Vertical::Center).width(Length::Fill).size(24)
        )
        .style(style::button::rounded_primary)
        .on_press(Message::NewChat)
        .width(Length::Fill).padding(Padding::from(10));

        let new_button = container(new_button).padding(Padding::from(10)).width(Length::Fill).align_x(Horizontal::Center).align_y(Vertical::Center);
        container(column![
            self.header("Chats".to_string()),
            new_button,
            self.view_chats(app),
            //container(app.main_view.chats.view(app.logic.chat)).height(Length::Fill),
            vertical_space(),
        ]).width(Length::FillPortion(10))
        .style(style::container::side_bar).into()
    }

    pub fn view_chats<'a>(&'a self, app : &'a ChatApp) -> Element<Message>{
        let chats_within = app.main_view.chats.clone();
        let chats_out = app.main_view.chats.clone();
        //chats_within.chats = chats_within.chats.into_iter().filter(|x| x.time.duration_since(SystemTime::now()).unwrap().as_secs() < 2629746).collect();
        //chats_out.chats = chats_out.chats.into_iter().filter(|x| x.time.duration_since(SystemTime::now()).unwrap().as_secs() > 2629746).collect();

        let txt = |title : String, color : iced::Color| -> Element<Message>{
            text(title)
            .color(color)
            .size(16)
            .width(Length::FillPortion(6))
            .align_y(Vertical::Center)
            .align_x(Horizontal::Center)
            .into()
        };
        if app.main_view.chats.chats.len() >= 8{
            let chosen = app.logic.chat.unwrap_or(usize::MAX);
            let view = |chats : Vec<&'a Chat>| -> Element<Message>{
                let chats : Vec<Element<Message>> = chats.iter().enumerate().map(|(i, x)| x.view(i == chosen)).clone().collect();
                return scrollable(column(chats).spacing(2)).into();
            };

            return column![
                txt("This Month".to_string(), self.theme().palette().primary),
                view((&app.main_view.chats.chats).iter().filter(|x| x.time.duration_since(SystemTime::now()).unwrap_or(Duration::new(0, 0)).as_secs() < 2629746).collect::<Vec<&Chat>>()),
                txt("Old".to_string(), self.theme().palette().primary),
                view((&app.main_view.chats.chats).iter().filter(|x| x.time.duration_since(SystemTime::now()).unwrap_or(Duration::new(0, 0)).as_secs() > 2629746).collect::<Vec<&Chat>>()),
                //container(chats_out.view(app.logic.chat)),
            ].into()
        }else{
            return column![
                txt("All".to_string(), self.theme().palette().primary),
                container(app.main_view.chats.view(app.logic.chat)),
            ].into()
        }
    }

}
