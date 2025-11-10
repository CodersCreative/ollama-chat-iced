use crate::PREVIEW_LEN;
use base64_stream::ToBase64Reader;
use iced::Color;
use image::ImageFormat;
#[cfg(feature = "voice")]
use rodio::{Decoder, OutputStream, Sink};
use std::{
    env,
    io::{self, Write},
};
use std::{
    error::Error,
    fs::{self, File},
    io::{BufReader, Cursor, Read},
    path::Path,
};
use text_splitter::TextSplitter;

#[cfg(feature = "voice")]
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
    let splitter = TextSplitter::new(len);

    let chunks = splitter.chunks(&text).collect::<Vec<&str>>();

    return chunks
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
}

pub fn generate_id() -> i32 {
    let num = rand::random_range(0..100000);
    return num;
}

pub fn split_text_into_thinking(text: String) -> (String, Option<String>) {
    if text.contains("<think>") {
        let c = text.clone();
        let split = c.split_once("<think>").unwrap();
        let mut content = split.0.to_string();
        let think = if split.1.contains("</think>") {
            let split2 = split.1.rsplit_once("</think>").unwrap();
            content.push_str(split2.1);
            split2.0.to_string()
        } else {
            split.1.to_string()
        };

        (
            content.trim().to_string(),
            if !think.trim().is_empty() {
                Some(think.trim().to_string())
            } else {
                None
            },
        )
    } else {
        (text, None)
    }
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

pub fn convert_image(path: &Path) -> Result<String, Box<dyn Error>> {
    let f = BufReader::new(File::open(path)?);

    let format = ImageFormat::from_path(path)?;
    if !matches!(format, ImageFormat::Png | ImageFormat::Jpeg) {
        let img = image::load(f, format)?;
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)?;
        let mut reader = ToBase64Reader::new(buf.as_slice());
        let mut base64 = String::new();
        reader.read_to_string(&mut base64)?;
        return Ok(base64);
    }

    let mut reader = ToBase64Reader::new(f);
    let mut base64 = String::new();
    reader.read_to_string(&mut base64)?;

    Ok(base64)
}

pub fn convert_audio(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let mut reader = ToBase64Reader::new(buffer.as_slice());
    let mut base64 = String::new();
    reader.read_to_string(&mut base64)?;
    Ok(base64)
}

pub fn lighten_colour(color: Color, amt: f32) -> Color {
    darken_colour(color, -amt)
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
    let colour: Vec<f32> = color.into_rgba8().into_iter().map(|x| x as f32).collect();
    let mut colour: Vec<f32> = rgb2hsl(&colour);
    colour[2] = (colour[2] - amt).min(1.0).max(0.0);
    let colour = hsl2rgb(&colour);
    return Color::from_rgba(
        colour[0] / 255.0,
        colour[1] / 255.0,
        colour[2] / 255.0,
        color.a.into(),
    );
}

// HSL and RGB Conversions from https://github.com/JiatLn/color-art/blob/main/src/conversion/hsl.rs

fn hsl2rgb(color: &[f32]) -> Vec<f32> {
    let h = color[0];
    let s = color[1];
    let l = color[2];

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = l - c / 2.0;

    let rgb = match h {
        h if (0.0..60.0).contains(&h) => vec![c, x, 0.0],
        h if (60.0..120.0).contains(&h) => vec![x, c, 0.0],
        h if (120.0..180.0).contains(&h) => vec![0.0, c, x],
        h if (180.0..240.0).contains(&h) => vec![0.0, x, c],
        h if (240.0..300.0).contains(&h) => vec![x, 0.0, c],
        h if (300.0..360.0).contains(&h) => vec![c, 0.0, x],
        _ => panic!(),
    };

    rgb.iter().map(|x| (x + m) * 255.0).collect()
}

fn rgb2hsl(color: &[f32]) -> Vec<f32> {
    let color: Vec<f32> = color.into_iter().map(|x| x / 255.0).collect();
    let r = color[0];
    let g = color[1];
    let b = color[2];

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);

    let mut h = 0.0;
    let mut s = 0.0;
    let l = (max + min) / 2.0;

    let delta = max - min;

    if delta != 0.0 {
        h = match max {
            x if x == r => 60.0 * (((g - b) / delta) % 6.0),
            x if x == g => 60.0 * ((b - r) / delta + 2.0),
            x if x == b => 60.0 * ((r - g) / delta + 4.0),
            _ => panic!(),
        };

        if h < 0.0 {
            h += 360.0;
        }

        s = delta / (1.0 - (2.0 * l - 1.0).abs());
        s = s.max(0.0).min(1.0);
    }

    vec![h, s, l]
}
