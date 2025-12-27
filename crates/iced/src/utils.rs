use iced::Color;
#[cfg(feature = "voice")]
use rodio::{Decoder, OutputStream, Sink};
use std::fmt::Display;

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

pub fn get_path_dir(path: String) -> String {
    let mut new_path = env!("CARGO_MANIFEST_DIR").to_string();
    new_path.push_str(&format!("/{}", path));
    new_path
}

pub fn write_read(message: String) -> String {
    println!("{}", message);
    return read_input();
}

pub fn get_path_src(path: String) -> String {
    get_path_dir(format!("src/{}", path))
}

pub fn get_path_assets<T: Display>(path: T) -> String {
    get_path_dir(format!("assets/{}", path))
}

pub fn generate_id() -> i32 {
    let num = rand::random_range(0..100000);
    return num;
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
