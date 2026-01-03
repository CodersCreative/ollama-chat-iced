use iced::{
    Task,
    futures::StreamExt,
    task::{Straw, sipper},
};
use ochat_types::generation::tts::TtsResponse;
use std::{
    fmt::Debug,
    rc::Rc,
    sync::{Arc, atomic::AtomicBool},
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
        let mut response = ochat_common::audio::play(data, Arc::new(AtomicBool::new(true))).await;

        while let Some(data) = response.next().await {
            if let ochat_common::audio::PlayerStreamResult::Err(e) = data {
                let _ = output.send(PlayerState::Err(e));
            }
        }

        let _ = output.send(PlayerState::Finished).await;

        Ok(())
    })
}
