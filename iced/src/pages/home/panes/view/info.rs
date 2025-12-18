use iced::{
    Element, Padding,
    alignment::{Horizontal, Vertical},
    widget::{button, center, column, container, image, row, rule, scrollable, text},
};

use crate::{
    Application, Message,
    font::{BODY_SIZE, HEADER_SIZE, SUB_HEADING_SIZE, get_bold_font},
    style,
    utils::get_path_assets,
};

pub fn view<'a>(app: &'a Application) -> Element<'a, Message> {
    let sub_heading = |txt: &'static str| text(txt).size(BODY_SIZE).style(style::text::primary);
    let banner = text("ochat")
        .font(get_bold_font())
        .size(HEADER_SIZE)
        .style(style::text::primary);

    let iced_version = text(format!("ochat-iced version: {}", app.cache.versions.iced))
        .size(BODY_SIZE)
        .style(style::text::text);

    let server_version = text(format!(
        "ochat-server version: {}",
        app.cache.versions.server
    ))
    .size(BODY_SIZE)
    .style(style::text::text);

    let latest_version = text(format!(
        "latest ochat version: {}",
        app.cache.versions.latest
    ))
    .size(BODY_SIZE)
    .style(style::text::text);

    let crvcode_img = image(image::Handle::from_path(get_path_assets(
        "crvcode.png".to_string(),
    )))
    .width(200)
    .height(200);

    let yt_btn = button(text("YouTube").style(style::text::text).size(BODY_SIZE))
        .style(style::button::transparent_back_white_text)
        .padding(10)
        .on_press(Message::UriClicked(String::from("youtube.com/@crvcode")));

    let gh_btn = button(text("GitHub").style(style::text::text).size(BODY_SIZE))
        .style(style::button::transparent_back_white_text)
        .padding(10)
        .on_press(Message::UriClicked(String::from(
            "github.com/CodersCreative",
        )));

    let crvcode_text = text("Creative Coders")
        .size(SUB_HEADING_SIZE)
        .style(style::text::text);

    let me_text = text("Taida Chinamo")
        .size(BODY_SIZE)
        .style(style::text::text);

    center(
        container(
            scrollable::Scrollable::new(
                column![
                    banner,
                    rule::horizontal(1),
                    sub_heading("Created by:"),
                    crvcode_text,
                    me_text,
                    crvcode_img,
                    row![yt_btn, gh_btn]
                        .align_y(Vertical::Center)
                        .spacing(10)
                        .padding(0),
                    sub_heading("Versions"),
                    latest_version,
                    iced_version,
                    server_version
                ]
                .align_x(Horizontal::Center)
                .spacing(5),
            )
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::default(),
            )),
        )
        .max_width(800)
        .padding(Padding::new(20.0))
        .style(style::container::neutral_back),
    )
    .into()
}
