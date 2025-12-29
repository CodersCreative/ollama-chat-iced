use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use iced::{
    Task,
    task::{Straw, sipper},
};
use ochat_common::data::RequestType;
use ochat_types::{
    generation::stt::{SttQueryData, SttResponse},
    settings::SettingsProvider,
};
use std::{
    fmt::Debug,
    rc::Rc,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use voice_activity_detector::VoiceActivityDetector;

use crate::{Application, DATA, Message};

const SILENCE_THRESHOLD: usize = 20;
const VAD_THRESHOLD: f32 = 0.4;
#[derive(Debug, Clone)]
pub struct Recorder {
    pub on_finish: RecorderFinish,
    pub model: Option<SettingsProvider>,
    pub state: RecorderState,
}

#[derive(Clone)]
pub struct RecorderFinish(pub Rc<dyn Fn(&mut Application, Option<String>) -> Task<Message>>);

unsafe impl Send for RecorderFinish {}
impl Debug for RecorderFinish {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Recorder Finish Fn")
    }
}

#[derive(Debug, Clone)]
pub enum RecorderState {
    Idle,
    Recording,
    Generating,
    Completed(String),
    Finished,
    Err(String),
}

pub enum RecorderUpdate {
    Generating(RecorderState),
    Finished(Result<(), String>),
}

impl Recorder {
    pub fn new(on_finish: RecorderFinish, model: Option<SettingsProvider>) -> Self {
        Self {
            on_finish,
            model,
            state: RecorderState::Idle,
        }
    }

    pub fn start(&mut self) -> Task<RecorderUpdate> {
        match self.state {
            RecorderState::Err(_) | RecorderState::Finished | RecorderState::Idle => {
                let (task, _handle) = Task::sip(
                    record_stream(self.model.clone()),
                    RecorderUpdate::Generating,
                    RecorderUpdate::Finished,
                )
                .abortable();

                self.state = RecorderState::Recording;

                task
            }
            _ => Task::none(),
        }
    }

    pub fn progress(&mut self, progress: RecorderState) {
        self.state = progress;
    }
}

pub fn record_stream(model: Option<SettingsProvider>) -> impl Straw<(), RecorderState, String> {
    let req = DATA.read().unwrap().to_request();
    sipper(async move |mut output| {
        let recorded_data = Arc::new(Mutex::new(Vec::<f32>::new()));
        let is_running = Arc::new(AtomicBool::new(true));
        let processing_buffer = Arc::new(Mutex::new(Vec::<f32>::new()));

        let mut vad = VoiceActivityDetector::builder()
            .sample_rate(16000)
            .chunk_size(512 as usize)
            .build()
            .map_err(|e| e.to_string())?;

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

        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if !running_clone.load(Ordering::SeqCst) {
                        return;
                    }

                    let mut buffer = buf_clone.lock().unwrap();
                    buffer.extend_from_slice(data);

                    while buffer.len() >= 512 {
                        let chunk: Vec<f32> = buffer.drain(0..512).collect();

                        let i16_chunk: Vec<i16> = chunk
                            .iter()
                            .map(|&s| (s * i16::MAX as f32) as i16)
                            .collect();

                        let probability = vad.predict(i16_chunk);

                        if probability < VAD_THRESHOLD {
                            consecutive_silence_chunks += 1;
                        } else {
                            consecutive_silence_chunks = 0;
                        }

                        if consecutive_silence_chunks >= SILENCE_THRESHOLD {
                            running_clone.store(false, Ordering::SeqCst);
                            break;
                        }

                        rec_clone.lock().unwrap().extend_from_slice(&chunk);
                    }
                },
                |err| eprintln!("{}", err),
                None,
            )
            .map_err(|e| e.to_string())?;

        stream.play().map_err(|e| e.to_string())?;

        let _ = output.send(RecorderState::Recording).await;

        while is_running.load(Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        let _ = output.send(RecorderState::Generating).await;

        let final_data = SttQueryData {
            model,
            data: recorded_data.lock().unwrap().clone(),
            spec: ochat_types::generation::SoundSpec { sample_rate: 16000 },
        };

        let text = req
            .make_request::<SttResponse, SttQueryData>(
                &"generation/stt/run/",
                &final_data,
                RequestType::Get,
            )
            .await?
            .text;

        let _ = output.send(RecorderState::Completed(text)).await;

        Ok(())
    })
}
