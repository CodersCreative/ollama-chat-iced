use serde::{Deserialize, Serialize};

pub mod stt;
pub mod text;
pub mod tts;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SoundSpec {
    pub sample_rate: u32,
    // pub bits_per_sample: u16,
}
