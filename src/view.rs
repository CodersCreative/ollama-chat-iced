use iced::{
    widget::{column, combo_box, container, mouse_area, pick_list, row, scrollable, text, text_input}, Element, Length, Renderer, Theme
};
use crate::{utils::darken_colour, ChatApp, Message};
use crate::sidebar::chats::Chats as SideChats;

pub struct View{
    pub theme: Theme,
    pub loading : bool,
    pub input : String,
    pub indent : String,
    pub chats: SideChats,
}

impl View{
    pub fn new() -> Self{
        Self{
            theme : Theme::CatppuccinMocha,
            loading: false,
            indent: 8.to_string(),
            input: String::new(),
            chats: SideChats::new(Vec::new()),
        }
    }
    pub fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }
    
    pub fn chat_view<'a>(&'a self, app : &'a ChatApp) -> Element<Message>{
        let input : Element<Message> = match self.loading {
            false => {
                text_input::<Message, Theme, Renderer>("Enter your message", &self.input)
                .on_input(Message::Edit)
                .on_submit(Message::Submit)
                .width(Length::FillPortion(19)).into()
            },
            true => {
                text("Awaiting Response").into()
            }
        };

        container(column![
            container(scrollable(app.save.view_chat(self.theme().palette())).width(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20),
            input,
        ]).width(Length::FillPortion(5)).into()
    }

    pub fn chat_side_bar<'a>(&'a self, app : &'a ChatApp) -> Element<Message>{
        let indent_text = text("Indent : ").width(Length::FillPortion(1))
            .horizontal_alignment(iced::alignment::Horizontal::Right)
            .vertical_alignment(iced::alignment::Vertical::Bottom);

        let indent_amt = text_input::<Message, Theme, Renderer>("0", &self.indent)
        .on_input(Message::Edit)
        .on_submit(Message::Submit)
        .width(Length::FillPortion(1));

        let indent = row![
            indent_text,
            indent_amt,
        ];

        let header = container(text("Chats").size(36)).style(container::Appearance{
            background : Some(iced::Background::Color(self.theme().palette().text)),
            text_color : Some(self.theme().palette().background),
            ..Default::default()
        }).width(Length::Fill);

        let new_button = container(
            mouse_area(
                text("+")
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .width(Length::Fill).size(25))
            .on_press(Message::NewChat)
        )
        .width(Length::Fill)
        .style(container::Appearance{
            background : Some(iced::Background::Color(self.theme().palette().primary)),
            text_color: Some(self.theme().palette().background),
            ..Default::default()
        });
        
        container(column![
            header,
            container(app.main_view.chats.view(app.logic.chat, self.theme().palette().background)).height(Length::Fill),
            indent,
            pick_list(
                Theme::ALL,
                Some(self.theme()),
                Message::ChangeTheme,
            )
            .width(Length::Fill),
            container(column![
                container(combo_box(&app.logic.models, app.save.ai_model.as_str(), None, Message::ChangeModel)),
                new_button,
            ]),
        ]).width(Length::FillPortion(1)).style(container::Appearance{
            background : Some(iced::Background::Color(darken_colour(self.theme().palette().background.clone(), 0.01))),
            ..Default::default()
        }).into() 
    }
}
