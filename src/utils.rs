use crate::{save::chats::SavedChats, PREVIEW_LEN};
use base64_stream::ToBase64Reader;
use color_art::Color as Colour;
use iced::Color;
use image::ImageFormat;
use ollama_rs::generation::images::Image;
use rand::Rng;
use std::{
    env,
    io::{self, Write},
    time::SystemTime,
};
use std::{
    error::Error,
    fs::{self, File},
    io::{BufReader, Cursor, Read},
    path::Path,
};
use text_splitter::TextSplitter;

use rodio::{Decoder, OutputStream, Sink};

pub fn play_wav_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;
    let file = BufReader::new(File::open(path)?);
    let source = Decoder::new(file)?;
    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}

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

pub fn write_read_line(message: String) -> String {
    print!("{}", message);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    return input;
}

pub fn get_path_settings(path: String) -> String {
    let mut new_path = env::var("XDG_CONFIG_HOME")
        .or_else(|_| env::var("HOME"))
        .unwrap();
    new_path.push_str(&format!("/.config/ochat"));

    if !fs::exists(&new_path).unwrap_or(true) {
        fs::create_dir(&new_path).unwrap();
    }

    new_path.push_str(&format!("/{}", path));
    return new_path;
}

pub fn get_path_src(path: String) -> String {
    get_path_dir(format!("src/{}", path))
}

pub fn get_path_assets(path: String) -> String {
    get_path_dir(format!("assets/{}", path))
}

pub fn get_path_dir(path: String) -> String {
    let mut new_path = env!("CARGO_MANIFEST_DIR").to_string();
    new_path.push_str(&format!("/{}", path));
    return new_path;
}

pub fn split_text_gtts(text: String) -> Vec<String> {
    split_text_with_len(100, text)
}

pub fn split_text(text: String) -> Vec<String> {
    split_text_with_len(PREVIEW_LEN, text)
}

pub fn split_text_with_len(len: usize, text: String) -> Vec<String> {
    let splitter = TextSplitter::default().with_trim_chunks(true);

    let chunks = splitter.chunks(&text, len).collect::<Vec<&str>>();

    return chunks
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
}

pub fn generate_id() -> i32 {
    let num = rand::thread_rng().gen_range(0..100000);
    return num;
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
        let img = image::load(f, format)?;
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)?;
        let mut reader = ToBase64Reader::new(buf.as_slice());
        let mut base64 = String::new();
        reader.read_to_string(&mut base64)?;
        return Ok(Image::from_base64(&base64));
    }

    let mut reader = ToBase64Reader::new(f);
    let mut base64 = String::new();
    reader.read_to_string(&mut base64)?;
    Ok(Image::from_base64(&base64))
}

pub fn get_preview(chat: &SavedChats) -> (String, SystemTime) {
    if !chat.0.is_empty() {
        if chat.0.len() > 1 {
            let i = chat.0.len() - 2;
            let prev = split_text(chat.0[i].content().to_string());
            if prev.len() > 0 {
                return (prev[0].clone(), chat.3);
            }
        }
    }

    (String::from("New"), SystemTime::now())
}

pub fn lighten_colour(color: Color, amt: f32) -> Color {
    let colour = color.into_rgba8();
    let colour = Colour::from_rgba(colour[0], colour[1], colour[2], color.a.into())
        .unwrap()
        .lighten(amt.into());
    return Color::from_rgba(
        colour.red() as f32 / 255.0,
        colour.green() as f32 / 255.0,
        colour.blue() as f32 / 255.0,
        colour.alpha() as f32,
    );
}

pub fn change_alpha(color: Color, amt: f32) -> Color {
    let colour = color.into_rgba8();
    return Color::from_rgba(
        colour[0] as f32 / 255.0,
        colour[1] as f32 / 255.0,
        colour[2] as f32 / 255.0,
        amt as f32,
    );
}

pub fn darken_colour(color: Color, amt: f32) -> Color {
    let colour = color.into_rgba8();
    let colour = Colour::from_rgba(colour[0], colour[1], colour[2], color.a.into())
        .unwrap()
        .darken(amt.into());
    return Color::from_rgba(
        colour.red() as f32 / 255.0,
        colour.green() as f32 / 255.0,
        colour.blue() as f32 / 255.0,
        colour.alpha() as f32,
    );
}
