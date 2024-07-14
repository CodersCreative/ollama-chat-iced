use iced::{theme::Palette, widget::{column, container, text, mouse_area}};
use iced::{Background, Border, Element, Length};
use regex::Regex;
use serde::{Deserialize, Serialize};
use crate::{utils::darken_colour, Message};

#[derive(Serialize, Deserialize, Debug,Clone, PartialEq)]
pub struct Chat{
    pub name: String,
    pub message: String,
}
impl Chat{
    pub fn new(name : &str, messasge : &str) -> Self{
        return Self{
            name: name.to_string(),
            message: messasge.to_string()
        }
    }

    pub fn view(&self, palette : Palette, indent : usize) -> Element<Message> {
        let is_ai = self.name != "User";
        let accent = match is_ai{
            true => palette.danger,
            false => palette.primary,
        };

        let mut space = String::new();
        for i in 0..indent{
            space.push_str(" ");
        }

        let name = container(text(&self.name).size(16)).style(container::Appearance{
            background: Some(Background::Color(accent)),
            border: Border::with_radius(5),
            text_color: Some(palette.background),
            ..Default::default()
        }).width(Length::Fill).padding(3);
        
        let replace_spaces_with_tabs = |text: &str|-> String {
          let re = Regex::new(r"(?m)^[ ]+").unwrap();
          re.replace_all(text, space.as_str()).to_string()
        };
        
        let messagesplit = self.message.split_terminator("```");
        let mut messages = Vec::new();
        for (i, x) in messagesplit.enumerate(){
            if i % 2 != 0{
                let (l, c) = x.split_once("\n").unwrap();
                println!("{:?}", &c);
                let bg = palette.background;
                let c = replace_spaces_with_tabs(c);
                println!("{:?}", &c);

                let code = mouse_area(container(text(&c).size(18)).padding(8).style(container::Appearance{
                    background : Some(iced::Background::Color(darken_colour(bg, 0.02))),
                    border : Border::with_radius(5),
                    ..Default::default()
                }).width(Length::Fill)).on_press(Message::SaveToClipboard(c.to_string()));

                let lang = container(text(l).size(16)).padding(8).style(container::Appearance{
                    background : Some(iced::Background::Color(darken_colour(bg, 0.03))),
                    border : Border::with_radius(5),
                    ..Default::default()
                }).width(Length::Fill);


                let tip = container(text("Click to copy.").size(12)).padding(6).style(container::Appearance{
                    background : Some(iced::Background::Color(darken_colour(bg, 0.03))),
                    border : Border::with_radius(5),
                    ..Default::default()
                }).width(Length::Fill);

                let code_snippet = column![
                    lang,
                    code,
                    tip,
                ];

                messages.push(code_snippet.into());
                println!("y");
            }else{
                messages.push(text(x).size(18).into());
                println!("n");
            }
        }
        let mcontainer = container(column(messages)).padding(8);
        
        let ret_message = container(column![name,mcontainer,].width(Length::Fill)).style(container::Appearance{
            border: Border{color: accent, width: 2.0, radius: 5.into()},
            ..Default::default()
        }).width(Length::FillPortion(5));

        let adjusted = ret_message;
        container(adjusted).into()
    }
}
