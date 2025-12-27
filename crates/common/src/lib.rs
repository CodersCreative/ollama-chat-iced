use base64_stream::ToBase64Reader;
use image::ImageFormat;
use ochat_types::user::Token;
use serde::de::DeserializeOwned;
use std::{
    env,
    error::Error,
    fs::{self, File},
    io::{BufReader, Cursor, Read},
    path::Path,
};

pub mod data;

const TOKEN_PATH: &str = "jwt.json";

pub fn save_token(token: &Token) {
    let path = get_path_settings(TOKEN_PATH.to_string());
    let writer = File::create(path);

    if let Ok(writer) = writer {
        let _ = serde_json::to_writer_pretty(writer, &token);
    }
}

pub fn load_token() -> Result<Token, String> {
    let path = get_path_settings(TOKEN_PATH.to_string());
    load_from_file(&path)
}

pub fn print_param_count(params: &u64) -> String {
    match params.ilog10() {
        0..3 => format!("{}", params),
        3..6 => format!("{}K", params / 1000),
        6..9 => format!("{}M", params / 1_000_000),
        9..12 => format!("{}G", params / 1_000_000_000),
        _ => format!("{}T", params / 1_000_000_000_000),
    }
}

pub fn print_data_size(size: &u64) -> String {
    match size.ilog10() {
        0..3 => format!("{} B", size),
        3..6 => format!("{} KB", size / 1000),
        6..9 => format!("{} MB", size / 1_000_000),
        9..12 => format!("{} GB", size / 1_000_000_000),
        _ => format!("{} TB", size / 1_000_000_000_000),
    }
}
pub fn load_from_file<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let reader = File::open(path);

    if let Ok(mut reader) = reader {
        let mut data = String::new();
        let _ = reader
            .read_to_string(&mut data)
            .map_err(|e| e.to_string())?;

        let de_data = serde_json::from_str(&data);

        return match de_data {
            Ok(x) => Ok(x),
            Err(e) => Err(e.to_string()),
        };
    }

    Err("Failed to open file".to_string())
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
    new_path
}

pub fn get_path_local(path: String) -> String {
    let mut new_path = env::var("XDG_CONFIG_HOME")
        .or_else(|_| env::var("HOME"))
        .unwrap();
    new_path.push_str(&format!("/.local/share/ochat"));

    if !fs::exists(&new_path).unwrap_or(true) {
        fs::create_dir(&new_path).unwrap();
    }

    new_path.push_str(&format!("/{}", path));
    new_path
}

pub fn get_path_dir(path: String) -> String {
    let mut new_path = env!("CARGO_MANIFEST_DIR").to_string();
    new_path.push_str(&format!("/{}", path));
    new_path
}

pub fn convert_image_to_b64(path: &Path) -> Result<String, Box<dyn Error>> {
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

pub fn convert_file_to_b64(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let mut reader = ToBase64Reader::new(buffer.as_slice());
    let mut base64 = String::new();
    reader.read_to_string(&mut base64)?;
    Ok(base64)
}

pub fn convert_audio_to_b64(path: &Path) -> Result<String, Box<dyn Error>> {
    convert_file_to_b64(path)
}
