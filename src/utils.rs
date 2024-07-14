use std::{io::{self, Write}, usize};
use iced::Color;
use color_art::Color as Colour;
use text_splitter::TextSplitter;
use crate::{save::chats::Chats, PREVIEW_LEN};

pub fn read_input() -> String {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    return input;
}

pub fn write_read(message: String) -> String {
    println!("{}", message);
    return read_input();
}

pub fn write_read_line(message: String) -> String{
    print!("{}", message);
    io::stdout().flush().unwrap();  // Flush to display the prompt
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    return input;
}

pub fn split_text(text: String) -> Vec<String> {
    let max_characters = PREVIEW_LEN.clone();

    let splitter = TextSplitter::default().with_trim_chunks(true);

    let chunks = splitter
        .chunks(&text, max_characters)
        .collect::<Vec<&str>>();

    return chunks.iter().map(|x| x.to_string()).collect::<Vec<String>>();
}

pub fn split_text_new_line(text: String) -> String {
    let split = split_text(text.clone());
    let mut t = String::new();
    for i in 0..split.len() {
        if i <= 0 {
            t.push_str(&split[i]);
        } else {
            let str = format!("\n{}", split[i].clone());
            t.push_str(&str);
        }
    }
    return t;
}

pub fn get_preview(chat: &Chats) -> String{
    if !chat.0.is_empty(){
        let i = chat.0.len() - 2;
        let prev = split_text(chat.0[i].message.clone().to_string());
        if prev.len() > 0{ 
            return prev[0].clone();
        }
    }

    String::from("New")
}

pub fn lighten_colour(color : Color, amt : f32) -> Color{
    let colour = color.into_rgba8();
    let colour = Colour::from_rgba(colour[0], colour[1], colour[2], color.a.into()).unwrap().lighten(amt.into());
    return Color::from_rgba(colour.red() as f32 / 255.0, colour.green() as f32 / 255.0, colour.blue() as f32 / 255.0, colour.alpha() as f32);
}
pub fn darken_colour(color : Color, amt : f32) -> Color{
    let colour = color.into_rgba8();
    let colour = Colour::from_rgba(colour[0], colour[1], colour[2], color.a.into()).unwrap().darken(amt.into());
    return Color::from_rgba(colour.red() as f32 / 255.0, colour.green() as f32 / 255.0, colour.blue() as f32 / 255.0, colour.alpha() as f32);
}
