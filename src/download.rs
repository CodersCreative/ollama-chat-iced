use crate::{common::Id, style, ChatApp, Message};
use iced::{
    alignment::{Horizontal, Vertical},
    futures::{SinkExt, Stream, StreamExt},
    stream::try_channel,
    widget::{button, column, container, progress_bar, text},
    Element, Length, Subscription,
};
use ollama_rs::Ollama;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct Download {
    download: String,
    pub state: State,
}

#[derive(Debug)]
pub enum State {
    Downloading(f32, String),
    Finished,
    Errored,
}

impl Download {
    pub fn new(download: String) -> Self {
        Download {
            download,
            state: State::Downloading(0.0, String::new()),
        }
    }

    pub fn progress(&mut self, new_progress: Result<DownloadProgress, String>) {
        if let State::Downloading(progress, message) = &mut self.state {
            match new_progress {
                Ok(DownloadProgress::Downloading(per, mes)) => {
                    *progress = per;
                    *message = mes;
                }
                Ok(DownloadProgress::Finished) => {
                    self.state = State::Finished;
                }
                Err(_) => {
                    self.state = State::Errored;
                }
            }
        }
    }

    pub fn subscription(&self, app: &ChatApp, id: Id) -> Subscription<Message> {
        match self.state {
            State::Downloading(_, _) => {
                pull(id, self.download.clone(), app.logic.ollama.clone()).map(Message::Pulling)
            }
            _ => Subscription::none(),
        }
    }

    fn txt<'a>(title: String, color: iced::Color) -> Element<'a, Message> {
        text(title)
            .color(color)
            .size(16)
            .width(Length::FillPortion(6))
            .align_y(Vertical::Center)
            .align_x(Horizontal::Center)
            .into()
    }

    pub fn view<'a>(&'a self, app: &ChatApp, id: Id) -> Element<'a, Message> {
        let (per, message) = match &self.state {
            State::Downloading(x, y) => (x.clone(), y.to_string()),
            _ => (100.0, String::new()),
        };

        let name = Self::txt(
            app.main_view.downloads().get(&id).unwrap().download.clone(),
            app.theme().palette().primary,
        );

        let bar = progress_bar(0.0..=100.0, per);
        let info = Self::txt(
            format!("Downloading... {per:.2}%"),
            app.theme().palette().danger,
        );
        let message = Self::txt(message, app.theme().palette().text);

        container(
            button(column![name, bar, info, message])
                .style(style::button::transparent_text)
                .on_press(Message::StopPull(id)),
        )
        .padding(10)
        .into()
    }
}

#[derive(Debug, Clone)]
pub enum DownloadProgress {
    Downloading(f32, String),
    Finished,
}

pub fn pull(
    id: Id,
    model: String,
    ollama: Arc<Mutex<Ollama>>,
) -> iced::Subscription<(Id, Result<DownloadProgress, String>)> {
    Subscription::run_with_id(
        id,
        download_stream(model, ollama).map(move |progress| (id, progress)),
    )
}

pub fn download_stream(
    model: String,
    ollama: Arc<Mutex<Ollama>>,
) -> impl Stream<Item = Result<DownloadProgress, String>> {
    try_channel(1, move |mut output| async move {
        let ollama = ollama.lock().await;
        let mut y = ollama
            .pull_model_stream(model, false)
            .await
            .map_err(|x| x.to_string())?;
        let mut total = 1;
        let mut completed = 0;
        let _ = output
            .send(DownloadProgress::Downloading(0.0, String::new()))
            .await;

        while let Some(status) = y.next().await {
            let status = status.map_err(|x| x.to_string())?;
            if let Some(x) = status.total {
                total = x;
            }

            if let Some(x) = status.completed {
                completed = x;
            }
            let _ = output
                .send(DownloadProgress::Downloading(
                    completed as f32 / total as f32 * 100.0,
                    status.message,
                ))
                .await;
        }

        let _ = output.send(DownloadProgress::Finished).await;

        Ok(())
    })
}
