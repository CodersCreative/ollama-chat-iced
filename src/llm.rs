use crate::{common::Id, options::ModelOptions, tools::SavedTool, ChatApp, Message};
use iced::{
    futures::{SinkExt, Stream, StreamExt},
    stream::try_channel,
    Subscription,
};
use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage},
    Ollama,
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, usize};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum ChatProgress {
    Generating(ChatMessage),
    Finished,
}

pub fn get_model() -> Ollama {
    return Ollama::default();
}

pub async fn run_ollama_multi(
    chats: Vec<ChatMessage>,
    options: ModelOptions,
    ollama: Arc<Mutex<Ollama>>,
    saved_id: Id,
    index: usize,
) -> Result<(ChatMessage, Id, String, usize), String> {
    let o = ollama.lock().await;
    let model = options.model().to_string();

    let request = ChatMessageRequest::new(options.model().to_string(), chats.to_vec())
        .options(options.into());
    let result = o.send_chat_messages(request).await;

    if let Ok(result) = result {
        return Ok((result.message, saved_id, model, index));
    }

    return Err("Failed to run ollama.".to_string());
}

pub async fn delete_model(ollama: Arc<Mutex<Ollama>>, model: String) {
    let o = ollama.lock().await;
    let _ = o.delete_model(model).await;
}

pub fn run_ollama_stream(
    chats: Arc<Vec<ChatMessage>>,
    tools: Arc<Vec<SavedTool>>,
    options: ModelOptions,
    ollama: Arc<Mutex<Ollama>>,
) -> impl Stream<Item = Result<ChatProgress, String>> {
    try_channel(1, |mut output| async move {
        let ollama = ollama.lock().await;
        let request = ChatMessageRequest::new(options.model().to_string(), chats.to_vec())
            .options(options.into())
            .tools(tools.to_vec().into_iter().map(|x| x.into()).collect());

        let mut y = ollama
            .send_chat_messages_stream(request)
            .await
            .map_err(|x| x.to_string())?;

        let _ = output
            .send(ChatProgress::Generating(ChatMessage {
                thinking: None,
                role: ollama_rs::generation::chat::MessageRole::Assistant,
                content: String::new(),
                images: None,
                tool_calls: Vec::new(),
            }))
            .await;

        while let Some(Ok(response)) = y.next().await {
            // let _ = response.message.tool_calls;
            let _ = output
                .send(ChatProgress::Generating(response.message))
                .await;
        }

        let _ = output.send(ChatProgress::Finished).await;

        Ok(())
    })
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChatStreamId(pub Id, pub usize, pub usize);

impl ChatStreamId {
    pub fn new(saved: Id, parent: usize, index: usize) -> Self {
        return Self(saved, parent, index);
    }
}

#[derive(Debug)]
pub struct ChatStream {
    pub state: State,
    pub chats: Arc<Vec<ChatMessage>>,
    pub options: ModelOptions,
    pub tools: Arc<Vec<SavedTool>>,
}

#[derive(Debug)]
pub enum State {
    Generating(ChatMessage),
    Finished,
    Errored,
}

pub fn chat(
    id: ChatStreamId,
    chats: Arc<Vec<ChatMessage>>,
    tools: Arc<Vec<SavedTool>>,
    options: ModelOptions,
    ollama: Arc<Mutex<Ollama>>,
) -> iced::Subscription<(ChatStreamId, Result<ChatProgress, String>)> {
    Subscription::run_with_id(
        id,
        run_ollama_stream(chats, tools, options, ollama).map(move |progress| (id, progress)),
    )
}

impl ChatStream {
    pub fn new(
        app: &ChatApp,
        chats: Arc<Vec<ChatMessage>>,
        tools: Arc<Vec<SavedTool>>,
        model: usize,
    ) -> Self {
        Self {
            state: State::Generating(ChatMessage::new(
                ollama_rs::generation::chat::MessageRole::Assistant,
                String::new(),
            )),
            chats: chats,
            options: app.options.model_options()[model].clone(),
            tools: tools,
        }
    }

    pub fn progress(&mut self, new_progress: Result<ChatProgress, String>) {
        if let State::Generating(message) = &mut self.state {
            match new_progress {
                Ok(ChatProgress::Generating(mes)) => {
                    *message = mes;
                }
                Ok(ChatProgress::Finished) => {
                    self.state = State::Finished;
                }
                Err(_) => {
                    self.state = State::Errored;
                }
            }
        }
    }

    pub fn subscription(&self, app: &ChatApp, id: ChatStreamId) -> Subscription<Message> {
        match self.state {
            State::Generating(_) => chat(
                id,
                self.chats.clone(),
                self.tools.clone(),
                self.options.clone(),
                app.logic.ollama.clone(),
            )
            .map(Message::Generating),
            _ => Subscription::none(),
        }
    }
}
