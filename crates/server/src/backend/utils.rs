use std::{env, fs};

use crate::backend::{CONN, errors::ServerError};

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

pub async fn get_count(query: &str) -> Result<u8, ServerError> {
    let mut count: u8 = 0;
    let query: Option<serde_json::Value> = CONN.query(query).await?.take(0)?;

    if let Some(mut query) = query {
        if query.is_array() {
            query = query[0].clone();
        }
        if query.is_object() {
            count = query["count"].as_number().unwrap().as_u64().unwrap() as u8;
        }
    }

    Ok(count)
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
    return new_path;
}

pub fn get_file_uploads_path(path: String) -> String {
    let mut new_path = get_path_local("uploads".to_string());

    if !fs::exists(&new_path).unwrap_or(true) {
        fs::create_dir(&new_path).unwrap();
    }

    new_path.push_str(&format!("/{}", path));
    return new_path;
}
