use std::{io::{self, Write}, time::SystemTime};
use iced::Color;
use color_art::Color as Colour;
use text_splitter::TextSplitter;
use crate::{save::chats::Chats, PREVIEW_LEN};
use base64_stream::ToBase64Reader;
use image::ImageFormat;
use ollama_rs::generation::images::Image;
use std::{
    fs::File,
    io::{BufReader, Cursor, Read},
    path::{Path, PathBuf},
    error::Error
}; 

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
pub fn convert_image(path: &Path) -> Result<Image, Box<dyn Error>> {
    let f = BufReader::new(File::open(path)?);

    let format = ImageFormat::from_path(path)?;
    if !matches!(format, ImageFormat::Png | ImageFormat::Jpeg) {
        //log::debug!("got {format:?} image, converting to png");
        let img = image::load(f, format)?;
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)?;
        let mut reader = ToBase64Reader::new(buf.as_slice());
        let mut base64 = String::new();
        reader.read_to_string(&mut base64)?;
        //log::debug!("converted to {} bytes of base64", base64.len());
        return Ok(Image::from_base64(&base64));
    }

    let mut reader = ToBase64Reader::new(f);
    let mut base64 = String::new();
    reader.read_to_string(&mut base64)?;
    //log::debug!("read image to {} bytes of base64", base64.len());
    Ok(Image::from_base64(&base64))
}
pub fn get_preview(chat: &Chats) -> (String, SystemTime){
    if !chat.0.is_empty(){
        let i = chat.0.len() - 2;
        let prev = split_text(chat.0[i].message.clone().to_string());
        if prev.len() > 0{ 
            return (prev[0].clone(), chat.2);
        }
    }

    (String::from("New"), SystemTime::now())
}

pub fn lighten_colour(color : Color, amt : f32) -> Color{
    let colour = color.into_rgba8();
    let colour = Colour::from_rgba(colour[0], colour[1], colour[2], color.a.into()).unwrap().lighten(amt.into());
    return Color::from_rgba(colour.red() as f32 / 255.0, colour.green() as f32 / 255.0, colour.blue() as f32 / 255.0, colour.alpha() as f32);
}

pub fn change_alpha(color : Color, amt : f32) -> Color{
    let colour = color.into_rgba8();
    return Color::from_rgba(colour[0] as f32 / 255.0, colour[1] as f32 / 255.0, colour[2] as f32 / 255.0, amt as f32);
}
pub fn darken_colour(color : Color, amt : f32) -> Color{
    let colour = color.into_rgba8();
    let colour = Colour::from_rgba(colour[0], colour[1], colour[2], color.a.into()).unwrap().darken(amt.into());
    return Color::from_rgba(colour.red() as f32 / 255.0, colour.green() as f32 / 255.0, colour.blue() as f32 / 255.0, colour.alpha() as f32);
}
