use crate::{Application, DATA, Message};
use iced::{
    Task,
    futures::StreamExt,
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
    sync::{Arc, atomic::AtomicBool},
};

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
        let mut recorded_data = Vec::new();
        let mut response = ochat_common::audio::record(Arc::new(AtomicBool::new(true)), true).await;

        while let Some(data) = response.next().await {
            match data {
                ochat_common::audio::RecorderStreamResult::Completed(x) => recorded_data = x,
                ochat_common::audio::RecorderStreamResult::Err(e) => return Err(e),
                _ => {}
            }
        }

        let _ = output.send(RecorderState::Generating).await;

        let final_data = SttQueryData {
            model,
            data: recorded_data,
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
