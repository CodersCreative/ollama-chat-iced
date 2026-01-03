use audio_io::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use futures::Stream;
use ochat_types::generation::{SoundSpec, tts::TtsResponse};
use std::{
    error::Error,
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use voice_activity_detector::VoiceActivityDetector;

use crate::convert_file_to_b64;

const SILENCE_THRESHOLD: usize = 20;
const VAD_THRESHOLD: f32 = 0.4;

#[derive(Debug, Clone)]
pub enum RecorderStreamResult {
    Recording(Vec<f32>),
    Completed(Vec<f32>),
    Err(String),
}

pub fn convert_audio_to_b64(path: &Path) -> Result<String, Box<dyn Error>> {
    convert_file_to_b64(path)
}

pub fn load_audio_file(path: &Path) -> Result<TtsResponse, Box<dyn Error>> {
    let data: AudioData<f32> = audio_read(path, AudioReadConfig::default())?;
    Ok(TtsResponse {
        spec: SoundSpec {
            sample_rate: data.sample_rate,
        },
        data: data.interleaved_samples,
    })
}

pub async fn record(
    is_running: Arc<AtomicBool>,
    use_vad: bool,
) -> impl Stream<Item = RecorderStreamResult> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        let recorded_data = Arc::new(Mutex::new(Vec::<f32>::new()));
        let processing_buffer = Arc::new(Mutex::new(Vec::<f32>::new()));

        let mut vad = VoiceActivityDetector::builder()
            .sample_rate(16000)
            .chunk_size(512 as usize)
            .build()
            .unwrap();

        let host = cpal::default_host();
        let device = host.default_input_device().expect("No input device");
        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: 16000,
            buffer_size: cpal::BufferSize::Default,
        };

        let mut consecutive_silence_chunks = 0;

        let rec_clone = Arc::clone(&recorded_data);
        let buf_clone = Arc::clone(&processing_buffer);
        let running_clone = Arc::clone(&is_running);
        let tx = Arc::new(tx);
        let tx_clone = Arc::clone(&tx);
        let start = Instant::now();

        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if !running_clone.load(Ordering::SeqCst) {
                        return;
                    }
                    let _ = tx_clone.send(RecorderStreamResult::Recording(data.to_vec()));

                    let mut buffer = buf_clone.lock().unwrap();
                    buffer.extend_from_slice(data);

                    while buffer.len() >= 512 {
                        let chunk: Vec<f32> = buffer.drain(0..512).collect();

                        let i16_chunk: Vec<i16> = chunk
                            .iter()
                            .map(|&s| (s * i16::MAX as f32) as i16)
                            .collect();

                        let probability = vad.predict(i16_chunk);

                        if probability < VAD_THRESHOLD && use_vad {
                            consecutive_silence_chunks += 1;
                        } else {
                            consecutive_silence_chunks = 0;
                        }

                        if consecutive_silence_chunks >= SILENCE_THRESHOLD
                            && start.elapsed() > Duration::from_secs(5)
                        {
                            running_clone.store(false, Ordering::SeqCst);
                            break;
                        }

                        rec_clone.lock().unwrap().extend_from_slice(&chunk);
                    }
                },
                |err| eprintln!("{}", err),
                None,
            )
            .unwrap();

        if let Err(e) = stream.play() {
            let _ = tx.send(RecorderStreamResult::Err(e.to_string()));
            return;
        };

        while is_running.load(Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        let _ = tx.send(RecorderStreamResult::Completed(
            recorded_data.lock().unwrap().clone(),
        ));
    });

    Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx))
}

#[derive(Debug, Clone)]
pub enum PlayerStreamResult {
    Playing(Vec<f32>),
    Finished,
    Err(String),
}

pub async fn play(
    data: TtsResponse,
    is_running: Arc<AtomicBool>,
) -> impl Stream<Item = PlayerStreamResult> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();

        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: data.spec.sample_rate,
            buffer_size: cpal::BufferSize::Default,
        };

        let sample_index = Arc::new(Mutex::new(0usize));
        let samples = Arc::new(data.data);

        let samples_clone = Arc::clone(&samples);
        let index_clone = Arc::clone(&sample_index);
        let tx = Arc::new(tx);
        let tx_clone = Arc::clone(&tx);

        let stream = match device.build_output_stream(
            &config,
            move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                if !is_running.load(Ordering::SeqCst) {
                    return;
                }
                let _ = tx_clone.send(PlayerStreamResult::Playing(output.to_vec()));
                let mut idx = index_clone.lock().unwrap();

                for frame in output.iter_mut() {
                    if *idx < samples_clone.len() {
                        *frame = samples_clone[*idx];
                        *idx += 1;
                    } else {
                        *frame = 0.0;
                    }
                }
            },
            |err| eprintln!("{}", err),
            None,
        ) {
            Ok(x) => x,
            Err(e) => {
                let _ = tx.send(PlayerStreamResult::Err(e.to_string()));
                return;
            }
        };

        if let Err(e) = stream.play() {
            let _ = tx.send(PlayerStreamResult::Err(e.to_string()));
            return;
        };

        while *sample_index.lock().unwrap() < samples.len() {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        let _ = tx.send(PlayerStreamResult::Finished);
    });

    Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx))
}
