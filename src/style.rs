pub mod container{
    use iced::{widget::container::Style, Theme};

    use crate::utils::{change_alpha, darken_colour};

    pub fn side_bar(theme: &Theme) -> Style{
        Style{
            background: Some(iced::Background::Color(darken_colour(theme.palette().background.clone(), 0.01))),
            ..Default::default()
        }
    }
    
    pub fn input_back(theme: &Theme) -> Style{
        Style{
            background: Some(iced::Background::Color(change_alpha(theme.palette().primary.clone(), 0.01))),
            border: iced::Border::default().width(10).color(change_alpha(theme.palette().primary.clone(), 0.05)).rounded(20),
            ..Default::default()
        }
    }

    pub fn chat_back(theme: &Theme) -> Style{
        Style{
            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
            border: iced::Border::default().width(2).color(theme.palette().primary.clone()).rounded(5),
            ..Default::default()
        }
    }

    pub fn chat_back_ai(theme: &Theme) -> Style{
        Style{
            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
            border: iced::Border::default().width(2).color(theme.palette().danger.clone()).rounded(5),
            ..Default::default()
        }
    }

    pub fn chat(theme: &Theme) -> Style{
        Style{
            background: Some(iced::Background::Color(theme.palette().primary.clone())),
            border: iced::Border::default().rounded(5),
            text_color: Some(theme.palette().text.clone()),
            ..Default::default()
        }
    }
    pub fn chat_ai(theme: &Theme) -> Style{
        Style{
            background: Some(iced::Background::Color(theme.palette().danger.clone())),
            border: iced::Border::default().rounded(5),
            text_color: Some(theme.palette().text.clone()),
            ..Default::default()
        }
    }
    
    pub fn code_darkened(theme: &Theme) -> Style{
        Style{
            background: Some(iced::Background::Color(darken_colour(theme.palette().background.clone(), 0.03))),
            border: iced::Border::default().rounded(5),
            ..Default::default()
        }
    }
    pub fn code(theme: &Theme) -> Style{
        Style{
            background: Some(iced::Background::Color(darken_colour(theme.palette().background.clone(), 0.02))),
            border: iced::Border::default().rounded(5),
            ..Default::default()
        }
    }
    pub fn bottom_input_back(theme: &Theme) -> Style{
        Style{
            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
            ..Default::default()
        }
    }
}

pub mod text_input{
    use iced::{widget::{text_input::{Style, Status}}, Theme};

    use crate::utils::change_alpha;

    pub fn input(theme: &Theme, _status: Status) -> Style{
        Style{
            border: iced::Border::default().rounded(0),
            background: iced::Background::Color(iced::Color::TRANSPARENT),
            placeholder: change_alpha(theme.palette().text.clone(), 0.4),
            value: theme.palette().text.clone(),
            selection: theme.palette().primary.clone(),
            icon: theme.palette().text.clone()
        }
    }
    pub fn ai_all(theme: &Theme, _status: Status) -> Style{
        Style{
            border: iced::Border::default().rounded(20),
            background: iced::Background::Color(change_alpha(theme.palette().background, 0.4)),
            placeholder: theme.palette().primary.clone(),
            value: theme.palette().primary.clone(),
            selection: theme.palette().primary.clone(),
            icon: theme.palette().text.clone()
        }
    }
}

pub mod button{
    use iced::{border::Radius, widget::{button::{Style, Status}}, Theme};
    use crate::utils::{change_alpha, darken_colour, lighten_colour};
    
    pub fn rounded_primary(theme: &Theme, _status: Status) -> Style{
        Style{
            background: Some(iced::Background::Color(theme.palette().primary.clone())),
            border: iced::Border::default().rounded(20),
            text_color: theme.palette().text.clone(),
            ..Default::default()
        }
    }

    pub fn start(theme: &Theme, _status: Status) -> Style{
        Style{
            background: Some(iced::Background::Color(lighten_colour(theme.palette().background.clone(), 0.05))),
            border: iced::Border::default().rounded(20).width(1).color(lighten_colour(theme.palette().background.clone(), 0.1)),
            text_color: change_alpha(theme.palette().text.clone(), 0.3),
            ..Default::default()
        }
    }
    
    pub fn start_chosen(theme: &Theme, _status: Status) -> Style{
        Style{
            background: Some(iced::Background::Color(change_alpha(theme.palette().primary.clone(), 0.05))),
            border: iced::Border::default().rounded(20).width(1).color(lighten_colour(theme.palette().primary.clone(), 0.1)),
            text_color: change_alpha(theme.palette().text.clone(), 0.3),
            ..Default::default()
        }
    }
    pub fn submit(theme: &Theme, _status: Status) -> Style{
        Style{
            background: Some(iced::Background::Color(theme.palette().primary.clone())),
            border: iced::Border::default().rounded(Radius::default().right(20)),
            text_color: theme.palette().text.clone(),
            ..Default::default()
        }
    }
    pub fn transparent_translucent(theme: &Theme, _status: Status) -> Style{
        Style{
            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
            text_color: change_alpha(lighten_colour(theme.palette().primary.clone(), 0.2), 0.4),
            ..Default::default()
        }
    }
    
    pub fn transparent_text(theme: &Theme, _status: Status) -> Style{
        Style{
            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
            text_color: theme.palette().text.clone(),
            ..Default::default()
        }
    }
    
    pub fn transparent_back(theme: &Theme, _status: Status) -> Style{
        Style{
            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
            text_color: theme.palette().background.clone(),
            ..Default::default()
        }
    }

    pub fn chosen_chat(theme: &Theme, _status: Status) -> Style{
        Style{
            background: Some(iced::Background::Color(theme.palette().background.clone())),
            text_color: theme.palette().text.clone(),
            ..Default::default()
        }
    }

    pub fn not_chosen_chat(theme: &Theme, _status: Status) -> Style{
        Style{
            background: Some(iced::Background::Color(darken_colour(theme.palette().background.clone(), 0.01))),
            text_color: change_alpha(theme.palette().text.clone(), 0.4),
            ..Default::default()
        }
    }
}
