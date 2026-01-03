use base64_stream::ToBase64Reader;
use enigo::{Keyboard, Settings};
use image::ImageFormat;
use ochat_types::user::Token;
use serde::de::DeserializeOwned;
use std::{
    env,
    error::Error,
    fs::{self, File},
    io::{BufReader, Cursor, Read},
    path::Path,
    sync::{LazyLock, RwLock},
};

#[cfg(feature = "sound")]
pub mod audio;
pub mod data;

static ENIGO: LazyLock<RwLock<Option<enigo::Enigo>>> =
    LazyLock::new(|| match enigo::Enigo::new(&Settings::default()) {
        Ok(x) => RwLock::new(Some(x)),
        _ => RwLock::new(None),
    });
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
    if params <= &0 {
        return String::from("Unknown");
    }
    match params.ilog10() {
        0..3 => format!("{}", params),
        3..6 => format!("{}K", params / 1000),
        6..9 => format!("{}M", params / 1_000_000),
        9..12 => format!("{}G", params / 1_000_000_000),
        _ => format!("{}T", params / 1_000_000_000_000),
    }
}

pub fn print_data_size(size: &u64) -> String {
    if size <= &0 {
        return String::from("Unknown");
    }
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

pub fn force_paste_text(text: &str) -> Result<(), String> {
    match &mut *ENIGO.write().unwrap() {
        Some(x) => x.text(text).map_err(|e| e.to_string())?,
        _ => return Err(String::from("Failed to paste")),
    }

    Ok(())
}

pub mod combinations {
    use rdev::{EventType, Key};
    use std::{
        collections::HashMap,
        rc::Rc,
        sync::{LazyLock, RwLock},
    };

    pub fn prink_key(value: &rdev::Key) -> Option<String> {
        Some(
            match value {
                Key::Alt => "alt",
                Key::AltGr => "alt",
                Key::Backspace => "back",
                Key::CapsLock => "caps",
                Key::ControlLeft | Key::ControlRight => "ctrl",
                Key::Delete => "del",
                Key::UpArrow => "up",
                Key::DownArrow => "down",
                Key::LeftArrow => "left",
                Key::RightArrow => "right",
                Key::End => "end",
                Key::Escape => "esc",
                Key::F1 => "f1",
                Key::F2 => "f2",
                Key::F3 => "f3",
                Key::F4 => "f4",
                Key::F5 => "f5",
                Key::F6 => "f6",
                Key::F7 => "f7",
                Key::F8 => "f8",
                Key::F9 => "f9",
                Key::F10 => "f10",
                Key::F11 => "f11",
                Key::F12 => "f12",
                Key::Home => "home",
                Key::MetaLeft | Key::MetaRight => "meta",
                Key::PageUp => "page_up",
                Key::PageDown => "page_down",
                Key::Return => "enter",
                Key::ShiftLeft | Key::ShiftRight => "shift",
                Key::Space => "space",
                Key::Tab => "tab",
                Key::BackQuote => "backtick",
                Key::Num1 | Key::Kp1 => "1",
                Key::Num2 | Key::Kp2 => "2",
                Key::Num3 | Key::Kp3 => "3",
                Key::Num4 | Key::Kp4 => "4",
                Key::Num5 | Key::Kp5 => "5",
                Key::Num6 | Key::Kp6 => "6",
                Key::Num7 | Key::Kp7 => "7",
                Key::Num8 | Key::Kp8 => "8",
                Key::Num9 | Key::Kp9 => "9",
                Key::Num0 | Key::Kp0 => "0",
                Key::Minus => "minus",
                Key::Equal => "equal",
                Key::KeyA => "A",
                Key::KeyB => "B",
                Key::KeyC => "C",
                Key::KeyD => "D",
                Key::KeyE => "E",
                Key::KeyF => "F",
                Key::KeyG => "G",
                Key::KeyH => "H",
                Key::KeyI => "I",
                Key::KeyJ => "J",
                Key::KeyK => "K",
                Key::KeyL => "L",
                Key::KeyM => "M",
                Key::KeyN => "N",
                Key::KeyO => "O",
                Key::KeyP => "P",
                Key::KeyQ => "Q",
                Key::KeyR => "R",
                Key::KeyS => "S",
                Key::KeyT => "T",
                Key::KeyU => "U",
                Key::KeyV => "V",
                Key::KeyW => "W",
                Key::KeyX => "X",
                Key::KeyY => "Y",
                Key::KeyZ => "Z",
                Key::LeftBracket => "(",
                Key::RightBracket => ")",
                Key::SemiColon => "semi_colon",
                Key::Function => "fn",
                Key::KpReturn => "enter",
                Key::KpMinus => "minus",
                Key::KpPlus => "plus",
                Key::KpMultiply => "multiply",
                Key::KpDivide => "divide",
                Key::Comma => "comma",
                Key::Dot => "full_stop",
                Key::Slash => "slash",
                Key::Insert => "ins",
                _ => return None,
            }
            .to_string(),
        )
    }

    pub static LOOKING_FOR: LazyLock<RwLock<LookingFor>> =
        LazyLock::new(|| RwLock::new(LookingFor::default()));

    #[derive(Clone)]
    pub struct NewCombinationFn(Rc<dyn Fn(Vec<Key>)>);
    #[derive(Clone)]
    pub struct NewCombinationKeyPressedFn(Rc<dyn Fn(Key)>);
    #[derive(Clone)]
    pub struct CombinationPressedFn(Rc<dyn Fn()>);

    #[derive(Default)]
    pub struct LookingFor {
        last: Vec<Key>,
        funcs: HashMap<u32, LookingForFn>,
        counter: u32,
    }

    unsafe impl Send for LookingFor {}
    unsafe impl Sync for LookingFor {}

    #[derive(Clone)]
    enum LookingForFn {
        New {
            func: NewCombinationFn,
            key_pressed: NewCombinationKeyPressedFn,
            pressed: Vec<Key>,
            esc: Key,
        },
        Combination {
            func: CombinationPressedFn,
            combo: Vec<Key>,
        },
    }

    pub fn get_new_combination(
        func: NewCombinationFn,
        key_pressed: NewCombinationKeyPressedFn,
        esc: Key,
    ) -> u32 {
        let mut val = LOOKING_FOR.write().unwrap();
        let counter = val.counter;
        val.funcs.insert(
            counter,
            LookingForFn::New {
                func,
                esc,
                key_pressed,
                pressed: Vec::new(),
            },
        );
        val.counter += 1;
        counter
    }

    pub fn add_new_combination(func: CombinationPressedFn, combo: Vec<Key>) -> u32 {
        let mut val = LOOKING_FOR.write().unwrap();
        let counter = val.counter;
        val.funcs
            .insert(counter, LookingForFn::Combination { func, combo });
        val.counter += 1;
        counter
    }

    pub fn remove_combination(key: &u32) {
        let _ = LOOKING_FOR.write().unwrap().funcs.remove(key);
    }

    pub fn listen() {
        let _ = rdev::listen(|event| {
            if let EventType::KeyPress(event) = event.event_type {
                let mut state = LOOKING_FOR.write().unwrap();
                let mut remove = Vec::new();
                state.last.push(event);
                let last = state.last.clone();

                for (key, val) in &mut state.funcs {
                    match val {
                        LookingForFn::Combination { func, combo } if combo.len() <= last.len() => {
                            if combo[..] == last[last.len() - combo.len()..] {
                                func.0();
                            }
                        }
                        LookingForFn::New {
                            func, esc, pressed, ..
                        } if &event == esc => {
                            func.0(pressed.clone());
                            remove.push(key);
                        }
                        LookingForFn::New {
                            key_pressed,
                            pressed,
                            ..
                        } => {
                            pressed.push(event);
                            key_pressed.0(event);
                        }
                        _ => {}
                    }
                }
            }
        });
    }
}
