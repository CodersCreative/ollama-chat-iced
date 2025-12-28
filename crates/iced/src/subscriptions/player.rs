use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use iced::{
    Task,
    task::{Straw, sipper},
};
use ochat_types::generation::tts::TtsResponse;
use std::{
    fmt::Debug,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::{Application, Message};

#[derive(Debug, Clone)]
pub struct Player {
    pub data: TtsResponse,
    pub on_finish: PlayerFinish,
    pub state: PlayerState,
}

#[derive(Clone)]
pub struct PlayerFinish(pub Rc<dyn Fn(&mut Application) -> Task<Message>>);

unsafe impl Send for PlayerFinish {}

impl Debug for PlayerFinish {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Player Finish Fn")
    }
}

#[derive(Debug, Clone)]
pub enum PlayerState {
    Idle,
    Playing,
    Finished,
    Err(String),
}

pub enum PlayerUpdate {
    Playing(PlayerState),
    Finished(Result<(), String>),
}

impl Player {
    pub fn new(data: TtsResponse, on_finish: PlayerFinish) -> Self {
        Self {
            on_finish,
            data,
            state: PlayerState::Idle,
        }
    }

    pub fn start(&mut self) -> Task<PlayerUpdate> {
        match self.state {
            PlayerState::Err(_) | PlayerState::Finished | PlayerState::Idle => {
                let (task, _handle) = Task::sip(
                    play_stream(self.data.clone()),
                    PlayerUpdate::Playing,
                    PlayerUpdate::Finished,
                )
                .abortable();

                self.state = PlayerState::Playing;

                task
            }
            _ => Task::none(),
        }
    }

    pub fn progress(&mut self, progress: PlayerState) {
        self.state = progress;
    }
}

pub fn play_stream(data: TtsResponse) -> impl Straw<(), PlayerState, String> {
    sipper(async move |mut output| {
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

        let stream = device
            .build_output_stream(
                &config,
                move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
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
            )
            .map_err(|e| e.to_string())?;

        stream.play().map_err(|e| e.to_string())?;

        while *sample_index.lock().unwrap() < samples.len() {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        let _ = output.send(PlayerState::Finished).await;

        Ok(())
    })
}
