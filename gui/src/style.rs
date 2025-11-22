pub mod container {
    use crate::utils::{change_alpha, darken_colour};
    use iced::border::bottom;
    use iced::{Theme, widget::container::Style};

    pub fn side_bar(theme: &Theme) -> Style {
        Style {
            background: Some(iced::Background::Color(darken_colour(
                theme.palette().background.clone(),
                0.02,
            ))),
            ..Default::default()
        }
    }

    pub fn side_bar_darker(theme: &Theme) -> Style {
        Style {
            background: Some(iced::Background::Color(darken_colour(
                theme.palette().background.clone(),
                0.05,
            ))),
            ..Default::default()
        }
    }

    pub fn input_back(theme: &Theme) -> Style {
        Style {
            background: Some(iced::Background::Color(change_alpha(
                theme.palette().primary.clone(),
                0.01,
            ))),
            border: iced::Border::default()
                .width(1)
                .color(theme.palette().primary)
                .rounded(20),
            ..Default::default()
        }
    }

    pub fn chat_back(theme: &Theme) -> Style {
        Style {
            background: Some(iced::Background::Color(darken_colour(
                theme.palette().background.clone(),
                0.01,
            ))),
            border: iced::Border::default().rounded(5),
            ..Default::default()
        }
    }

    pub fn neutral_back(theme: &Theme) -> Style {
        Style {
            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
            border: iced::Border::default()
                .width(1)
                .color(change_alpha(theme.palette().text, 0.2))
                .rounded(5),
            ..Default::default()
        }
    }

    pub fn window_back(theme: &Theme) -> Style {
        Style {
            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
            ..Default::default()
        }
    }

    pub fn window_title_back(theme: &Theme) -> Style {
        Style {
            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
            border: iced::Border::default()
                .width(2)
                .color(darken_colour(theme.palette().background.clone(), 0.05))
                .rounded(10),
            ..Default::default()
        }
    }

    pub fn window_back_danger(theme: &Theme) -> Style {
        Style {
            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
            border: iced::Border::default()
                .width(2)
                .color(change_alpha(theme.palette().danger.clone(), 0.4))
                .rounded(10),
            ..Default::default()
        }
    }

    pub fn chat(theme: &Theme) -> Style {
        Style {
            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
            border: iced::Border::default()
                .rounded(5)
                .width(1)
                .color(theme.palette().primary),
            text_color: Some(theme.palette().text.clone()),
            ..Default::default()
        }
    }
    pub fn chat_ai(theme: &Theme) -> Style {
        Style {
            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
            border: iced::Border::default()
                .rounded(5)
                .width(1)
                .color(theme.palette().danger),
            text_color: Some(theme.palette().text.clone()),
            ..Default::default()
        }
    }

    pub fn transparent_line(theme: &Theme) -> Style {
        Style {
            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
            border: iced::Border::default()
                .rounded(bottom(1))
                .width(1)
                .color(theme.palette().primary),
            text_color: Some(theme.palette().text.clone()),
            ..Default::default()
        }
    }

    pub fn code_darkened(theme: &Theme) -> Style {
        Style {
            background: Some(iced::Background::Color(darken_colour(
                theme.palette().background.clone(),
                0.03,
            ))),
            border: iced::Border::default().rounded(5),
            ..Default::default()
        }
    }
    pub fn code(theme: &Theme) -> Style {
        Style {
            background: Some(iced::Background::Color(darken_colour(
                theme.palette().background.clone(),
                0.02,
            ))),
            border: iced::Border::default().rounded(5),
            ..Default::default()
        }
    }
    pub fn bottom_input_back(_theme: &Theme) -> Style {
        Style {
            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
            ..Default::default()
        }
    }
}

pub mod text {
    use iced::{Theme, widget::text::Style};

    macro_rules! text_style {
        ($iden:ident) => {
            pub fn $iden(theme: &Theme) -> Style {
                Style {
                    color: Some(theme.palette().$iden.clone()),
                }
            }
        };
    }

    text_style!(text);
    text_style!(background);
    text_style!(primary);
    text_style!(danger);

    pub mod translucent {
        use super::*;
        use crate::utils::change_alpha;

        macro_rules! text_style_translucent {
            ($iden:ident) => {
                pub fn $iden(theme: &Theme) -> Style {
                    Style {
                        color: Some(change_alpha(theme.palette().$iden.clone(), 0.6)),
                    }
                }
            };
        }

        text_style_translucent!(text);
        text_style_translucent!(background);
        text_style_translucent!(primary);
        text_style_translucent!(danger);
    }
}

pub mod text_input {
    use crate::utils::change_alpha;
    use iced::{
        Theme,
        widget::text_input::{Status, Style},
    };

    pub fn input(theme: &Theme, _status: Status) -> Style {
        Style {
            border: iced::Border::default().rounded(0),
            background: iced::Background::Color(iced::Color::TRANSPARENT),
            placeholder: change_alpha(theme.palette().text.clone(), 0.4),
            value: theme.palette().text.clone(),
            selection: theme.palette().primary.clone(),
            icon: theme.palette().text.clone(),
        }
    }
    pub fn ai_all(theme: &Theme, _status: Status) -> Style {
        Style {
            border: iced::Border::default().rounded(20),
            background: iced::Background::Color(change_alpha(theme.palette().background, 0.4)),
            placeholder: theme.palette().primary.clone(),
            value: theme.palette().primary.clone(),
            selection: theme.palette().primary.clone(),
            icon: theme.palette().text.clone(),
        }
    }
}

pub mod text_editor {
    use crate::utils::change_alpha;
    use iced::{
        Theme,
        widget::text_editor::{Status, Style},
    };

    pub fn input(theme: &Theme, _status: Status) -> Style {
        Style {
            border: iced::Border::default().rounded(0),
            //background: iced::Background::Color(theme.palette().primary.clone()),
            background: iced::Background::Color(iced::Color::TRANSPARENT),
            placeholder: change_alpha(theme.palette().text.clone(), 0.4),
            value: theme.palette().text.clone(),
            selection: theme.palette().primary.clone(),
            icon: theme.palette().text.clone(),
        }
    }
}

pub mod svg {
    use iced::{
        Theme,
        widget::svg::{Status, Style},
    };

    macro_rules! svg_style {
        ($iden:ident) => {
            pub fn $iden(theme: &Theme, _status: Status) -> Style {
                Style {
                    color: Some(theme.palette().$iden.clone()),
                    ..Default::default()
                }
            }
        };
    }

    svg_style!(text);
    svg_style!(background);
    svg_style!(primary);
    svg_style!(danger);
}

pub mod svg_input {
    use crate::{Message, utils::get_path_assets};
    use iced::{
        Element, Length, Renderer, Theme,
        alignment::Vertical,
        widget::{column, container, horizontal_rule, row, svg, text_input},
    };

    macro_rules! svg_input {
        ($iden:ident) => {
            pub fn $iden<'a>(
                svg_path: Option<String>,
                input: text_input::TextInput<'a, Message, Theme, Renderer>,
                size: u16,
            ) -> Element<'a, Message> {
                container(column![
                    if let Some(path) = svg_path {
                        container(row![
                            svg(svg::Handle::from_path(get_path_assets(path)))
                                .style(super::svg::$iden)
                                .width(Length::Fixed(size as f32)),
                            input.style(super::text_input::input).size(size)
                        ].align_y(Vertical::Center)
                        .spacing(5))
                    } else {
                        container(input.style(super::text_input::input).size(size))
                    },
                    horizontal_rule(1).style(super::rule::translucent::$iden)
                ])
                .into()
            }
        };
    }

    svg_input!(text);
    svg_input!(background);
    svg_input!(primary);
    svg_input!(danger);
}

pub mod rule {
    use iced::{Theme, border::Radius, widget::rule::Style};

    use crate::utils::darken_colour;

    pub fn side_bar_darker(theme: &Theme) -> Style {
        Style {
            color: darken_colour(theme.palette().background.clone(), 0.05),
            width: 2,
            radius: Radius::new(5),
            fill_mode: iced::widget::rule::FillMode::Full,
        }
    }

    macro_rules! rule_style {
        ($iden:ident) => {
            pub fn $iden(theme: &Theme) -> Style {
                Style {
                    color: theme.palette().$iden.clone(),
                    width: 1,
                    radius: Radius::new(5),
                    fill_mode: iced::widget::rule::FillMode::Full,
                }
            }
        };
    }

    rule_style!(text);
    rule_style!(background);
    rule_style!(primary);
    rule_style!(danger);

    pub mod translucent {
        use super::*;
        use crate::utils::change_alpha;

        macro_rules! rule_style_translucent {
            ($iden:ident) => {
                pub fn $iden(theme: &Theme) -> Style {
                    Style {
                        color: change_alpha(theme.palette().$iden.clone(), 0.4),
                        width: 1,
                        radius: Radius::new(5),
                        fill_mode: iced::widget::rule::FillMode::Full,
                    }
                }
            };
        }

        rule_style_translucent!(text);
        rule_style_translucent!(background);
        rule_style_translucent!(primary);
        rule_style_translucent!(danger);
    }
}

pub mod svg_button {
    use iced::{
        Length, Renderer, Theme,
        widget::{button, svg},
    };

    use crate::{Message, style::button::transparent_back_white_text, utils::get_path_assets};

    macro_rules! svg_button {
        ($iden:ident) => {
            pub fn $iden<'a>(
                path: &'a str,
                size: u16,
            ) -> button::Button<'a, Message, Theme, Renderer> {
                button::Button::new(
                    svg(svg::Handle::from_path(get_path_assets(path.to_string())))
                        .style(super::svg::$iden)
                        .width(Length::Fixed(size as f32)),
                )
                .style(transparent_back_white_text)
            }
        };
    }

    svg_button!(text);
    svg_button!(background);
    svg_button!(primary);
    svg_button!(danger);
}

pub mod button {
    use crate::utils::{change_alpha, darken_colour, lighten_colour};
    use iced::{
        Theme,
        border::Radius,
        widget::button::{Status, Style},
    };

    pub fn rounded_primary(theme: &Theme, status: Status) -> Style {
        match status {
            Status::Active => Style {
                background: Some(iced::Background::Color(theme.palette().primary.clone())),
                border: iced::Border::default().rounded(20),
                text_color: theme.palette().text.clone(),
                ..Default::default()
            },
            Status::Hovered => Style {
                background: Some(iced::Background::Color(theme.palette().primary.clone())),
                border: iced::Border::default().rounded(20),
                text_color: theme.palette().primary.clone(),
                ..Default::default()
            },
            _ => Style {
                background: Some(iced::Background::Color(theme.palette().primary.clone())),
                border: iced::Border::default().rounded(20),
                text_color: theme.palette().danger.clone(),
                ..Default::default()
            },
        }
    }

    pub fn rounded_primary_blend(theme: &Theme, status: Status) -> Style {
        match status {
            Status::Active => Style {
                background: Some(iced::Background::Color(change_alpha(
                    theme.palette().primary.clone(),
                    0.1,
                ))),
                border: iced::Border::default()
                    .rounded(20)
                    .width(1)
                    .color(change_alpha(theme.palette().primary.clone(), 0.3)),
                text_color: theme.palette().text.clone(),
                ..Default::default()
            },
            Status::Hovered => Style {
                background: Some(iced::Background::Color(change_alpha(
                    theme.palette().primary.clone(),
                    0.1,
                ))),
                border: iced::Border::default()
                    .rounded(20)
                    .width(1)
                    .color(change_alpha(theme.palette().primary.clone(), 0.5)),
                text_color: theme.palette().text.clone(),
                ..Default::default()
            },
            _ => Style {
                background: Some(iced::Background::Color(change_alpha(
                    theme.palette().primary.clone(),
                    0.1,
                ))),
                border: iced::Border::default()
                    .rounded(20)
                    .width(1)
                    .color(change_alpha(theme.palette().primary.clone(), 0.7)),
                text_color: theme.palette().text.clone(),
                ..Default::default()
            },
        }
    }

    pub fn start(theme: &Theme, status: Status) -> Style {
        match status {
            Status::Active => Style {
                background: Some(iced::Background::Color(lighten_colour(
                    theme.palette().background.clone(),
                    0.05,
                ))),
                border: iced::Border::default()
                    .rounded(20)
                    .width(1)
                    .color(lighten_colour(theme.palette().background.clone(), 0.1)),
                text_color: change_alpha(theme.palette().text.clone(), 0.3),
                ..Default::default()
            },
            Status::Hovered => Style {
                background: Some(iced::Background::Color(lighten_colour(
                    theme.palette().background.clone(),
                    0.05,
                ))),
                border: iced::Border::default()
                    .rounded(20)
                    .width(1)
                    .color(lighten_colour(theme.palette().background.clone(), 0.1)),
                text_color: change_alpha(theme.palette().text.clone(), 0.5),
                ..Default::default()
            },
            _ => Style {
                background: Some(iced::Background::Color(lighten_colour(
                    theme.palette().background.clone(),
                    0.05,
                ))),
                border: iced::Border::default()
                    .rounded(20)
                    .width(1)
                    .color(lighten_colour(theme.palette().background.clone(), 0.1)),
                text_color: change_alpha(theme.palette().text.clone(), 0.7),
                ..Default::default()
            },
        }
    }

    pub fn start_chosen(theme: &Theme, status: Status) -> Style {
        match status {
            Status::Active => Style {
                background: Some(iced::Background::Color(change_alpha(
                    theme.palette().primary.clone(),
                    0.05,
                ))),
                border: iced::Border::default()
                    .rounded(20)
                    .width(1)
                    .color(lighten_colour(theme.palette().primary.clone(), 0.1)),
                text_color: change_alpha(theme.palette().text.clone(), 0.3),
                ..Default::default()
            },
            Status::Hovered => Style {
                background: Some(iced::Background::Color(change_alpha(
                    theme.palette().primary.clone(),
                    0.05,
                ))),
                border: iced::Border::default()
                    .rounded(20)
                    .width(1)
                    .color(lighten_colour(theme.palette().primary.clone(), 0.1)),
                text_color: change_alpha(theme.palette().text.clone(), 0.5),
                ..Default::default()
            },
            _ => Style {
                background: Some(iced::Background::Color(change_alpha(
                    theme.palette().primary.clone(),
                    0.05,
                ))),
                border: iced::Border::default()
                    .rounded(20)
                    .width(1)
                    .color(lighten_colour(theme.palette().primary.clone(), 0.1)),
                text_color: change_alpha(theme.palette().text.clone(), 0.7),
                ..Default::default()
            },
        }
    }
    pub fn submit(theme: &Theme, status: Status) -> Style {
        match status {
            Status::Active => Style {
                background: Some(iced::Background::Color(theme.palette().primary.clone())),
                border: iced::Border::default().rounded(Radius::default().right(20)),
                text_color: theme.palette().text.clone(),
                ..Default::default()
            },
            Status::Hovered => Style {
                background: Some(iced::Background::Color(theme.palette().primary.clone())),
                border: iced::Border::default().rounded(Radius::default().right(20)),
                text_color: theme.palette().background.clone(),
                ..Default::default()
            },
            _ => Style {
                background: Some(iced::Background::Color(theme.palette().primary.clone())),
                border: iced::Border::default().rounded(Radius::default().right(20)),
                text_color: theme.palette().danger.clone(),
                ..Default::default()
            },
        }
    }
    pub fn transparent_translucent(theme: &Theme, status: Status) -> Style {
        match status {
            Status::Active => Style {
                background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                text_color: change_alpha(lighten_colour(theme.palette().primary.clone(), 0.2), 0.4),
                ..Default::default()
            },
            Status::Hovered => Style {
                background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                text_color: change_alpha(lighten_colour(theme.palette().primary.clone(), 0.4), 0.6),
                ..Default::default()
            },
            _ => Style {
                background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                text_color: change_alpha(lighten_colour(theme.palette().primary.clone(), 0.6), 0.8),
                ..Default::default()
            },
        }
    }

    pub fn transparent_back_white_text(theme: &Theme, status: Status) -> Style {
        match status {
            Status::Disabled => Style {
                background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                text_color: theme.palette().text.clone(),
                ..Default::default()
            },
            Status::Active => Style {
                background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                text_color: theme.palette().text.clone(),
                ..Default::default()
            },
            Status::Hovered => Style {
                background: Some(iced::Background::Color(change_alpha(
                    theme.palette().text,
                    0.1,
                ))),
                border: iced::Border::default()
                    .rounded(5)
                    .color(change_alpha(theme.palette().primary, 0.2)),
                text_color: theme.palette().primary.clone(),
                ..Default::default()
            },
            _ => Style {
                background: Some(iced::Background::Color(change_alpha(
                    theme.palette().primary,
                    0.1,
                ))),
                border: iced::Border::default()
                    .rounded(5)
                    .color(change_alpha(theme.palette().danger, 0.2)),
                text_color: theme.palette().danger.clone(),
                ..Default::default()
            },
        }
    }

    pub fn transparent_back_black_text(theme: &Theme, status: Status) -> Style {
        match status {
            Status::Active => Style {
                background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                text_color: theme.palette().background.clone(),
                ..Default::default()
            },
            Status::Hovered => Style {
                background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                text_color: theme.palette().primary.clone(),
                ..Default::default()
            },
            _ => Style {
                background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                text_color: theme.palette().danger.clone(),
                ..Default::default()
            },
        }
    }

    pub fn chosen_chat(theme: &Theme, status: Status) -> Style {
        match status {
            Status::Active => Style {
                background: Some(iced::Background::Color(theme.palette().background.clone())),
                text_color: theme.palette().text.clone(),
                ..Default::default()
            },
            Status::Hovered => Style {
                background: Some(iced::Background::Color(theme.palette().background.clone())),
                text_color: theme.palette().primary.clone(),
                ..Default::default()
            },
            _ => Style {
                background: Some(iced::Background::Color(theme.palette().background.clone())),
                text_color: theme.palette().danger.clone(),
                ..Default::default()
            },
        }
    }

    pub fn side_bar_chat(theme: &Theme, status: Status) -> Style {
        match status {
            Status::Active => Style {
                background: Some(iced::Background::Color(darken_colour(
                    theme.palette().background.clone(),
                    0.01,
                ))),
                text_color: theme.palette().text.clone(),
                ..Default::default()
            },
            Status::Hovered => Style {
                background: Some(iced::Background::Color(darken_colour(
                    theme.palette().background.clone(),
                    0.05,
                ))),
                text_color: theme.palette().text.clone(),
                ..Default::default()
            },
            _ => Style {
                background: Some(iced::Background::Color(darken_colour(
                    theme.palette().background.clone(),
                    0.05,
                ))),
                text_color: theme.palette().primary.clone(),
                ..Default::default()
            },
        }
    }

    pub fn not_chosen_chat(theme: &Theme, status: Status) -> Style {
        match status {
            Status::Active => Style {
                background: Some(iced::Background::Color(darken_colour(
                    theme.palette().background.clone(),
                    0.01,
                ))),
                text_color: change_alpha(theme.palette().text.clone(), 0.4),
                ..Default::default()
            },
            Status::Hovered => Style {
                background: Some(iced::Background::Color(darken_colour(
                    theme.palette().background.clone(),
                    0.05,
                ))),
                text_color: change_alpha(theme.palette().text.clone(), 0.6),
                ..Default::default()
            },
            _ => Style {
                background: Some(iced::Background::Color(darken_colour(
                    theme.palette().background.clone(),
                    0.05,
                ))),
                text_color: change_alpha(theme.palette().text.clone(), 0.8),
                ..Default::default()
            },
        }
    }
}

pub mod markdown {
    use iced::{Padding, Theme, advanced::text::Highlight, widget::markdown::Style};

    use crate::utils::darken_colour;

    pub fn main(theme: &Theme) -> Style {
        Style {
            inline_code_highlight: Highlight {
                background: iced::Background::Color(theme.palette().background.clone()),
                border: iced::Border::default().rounded(10),
            },
            inline_code_padding: Padding::new(10.0),
            inline_code_color: darken_colour(theme.palette().background.clone(), 0.01),
            link_color: theme.palette().primary.clone(),
        }
    }
}

pub mod pick_list {
    use iced::{
        Theme,
        widget::pick_list::{Status, Style},
    };

    use crate::utils::change_alpha;

    pub fn main(theme: &Theme, status: Status) -> Style {
        match status {
            Status::Active => Style {
                text_color: theme.palette().text.clone(),
                placeholder_color: change_alpha(theme.palette().text.clone(), 0.6),
                handle_color: theme.palette().text.clone(),
                background: iced::Background::Color(change_alpha(theme.palette().text, 0.1)),
                border: iced::Border::default()
                    .rounded(5)
                    .color(theme.palette().text.clone()),
            },
            Status::Hovered => Style {
                text_color: theme.palette().text.clone(),
                placeholder_color: change_alpha(theme.palette().text.clone(), 0.6),
                handle_color: theme.palette().primary.clone(),
                background: iced::Background::Color(change_alpha(theme.palette().primary, 0.1)),
                border: iced::Border::default()
                    .rounded(5)
                    .color(theme.palette().primary.clone()),
            },
            Status::Opened => Style {
                text_color: theme.palette().text.clone(),
                placeholder_color: change_alpha(theme.palette().text.clone(), 0.6),
                handle_color: theme.palette().primary.clone(),
                background: iced::Background::Color(change_alpha(theme.palette().primary, 0.25)),
                border: iced::Border::default()
                    .rounded(5)
                    .color(theme.palette().danger.clone()),
            },
        }
    }
}

pub mod menu {
    use iced::{Theme, overlay::menu::Style};

    use crate::utils::{change_alpha, darken_colour};

    pub fn main(theme: &Theme) -> Style {
        Style {
            background: iced::Background::Color(darken_colour(
                theme.palette().background.clone(),
                0.05,
            )),
            border: iced::Border::default()
                .rounded(5)
                .color(theme.palette().primary.clone()),
            text_color: theme.palette().text.clone(),
            selected_text_color: theme.palette().primary.clone(),
            selected_background: iced::Background::Color(change_alpha(
                theme.palette().primary,
                0.1,
            )),
        }
    }
}

//pub mod markdown{
//    use iced::widget::{
//        button, container, hover,
//        text,
//    };
//    use iced::Element;
//    use crate::Message;
//    use crate::markdown;
//
//    pub struct CustomViewer;
//
//    impl<'a> markdown::Viewer<'a, Message> for CustomViewer {
//        fn on_link_click(url: markdown::Url) -> Message {
//            Message::URLClicked(url)
//        }
//
//        fn code_block(
//            &self,
//            settings: markdown::Settings,
//            _language: Option<&'a str>,
//            code: &'a str,
//            lines: &'a [markdown::Text],
//        ) -> Element<'a, Message> {
//            let code_block =
//                markdown::code_block(settings, lines, Message::URLClicked);
//
//            let copy = button(text("Copy").size(12))
//                .padding(2)
//                .on_press_with(|| Message::SaveToClipboard(code.to_owned()))
//                .style(button::text);
//
//            hover(
//                code_block,
//                container(copy).style(container::dark)
//                    .padding(settings.spacing / 2),
//            )
//        }
//    }
//}
