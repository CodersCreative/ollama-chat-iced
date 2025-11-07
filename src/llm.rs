use crate::{chats::TooledOptions, common::Id, options::ModelOptions, ChatApp, Message};
use iced::{
    futures::{SinkExt, Stream, StreamExt},
    stream::try_channel,
    Subscription,
};
use ollama_rs::{
    coordinator::Coordinator,
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage},
        tools::implementations::{
            Calculator, DDGSearcher, Scraper, SerperSearchTool, StockScraper,
        },
    },
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tools {
    DuckDuckGo,
    Serper,
    Scraper,
    Finance,
    Calculator,
}

pub async fn run_ollama(
    chats: Vec<ChatMessage>,
    options: ModelOptions,
    ollama: Arc<Mutex<Ollama>>,
) -> Result<ChatMessage, String> {
    let o = ollama.lock().await;

    let request = ChatMessageRequest::new(options.model().to_string(), chats.to_vec())
        .options(options.into());
    let result = o.send_chat_messages(request).await;

    if let Ok(result) = result {
        return Ok(result.message);
    }

    return Err("Failed to run ollama.".to_string());
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

async fn get_coordinator(
    tooled: Arc<TooledOptions>,
    options: ModelOptions,
    ollama: Arc<Mutex<Ollama>>,
) -> (Coordinator<Vec<ChatMessage>>, Vec<ChatMessage>) {
    let ollama = ollama.lock().await;
    let mut coordinator = Coordinator::new(ollama.clone(), options.model().to_string(), Vec::new())
        .options(options.into());

    let tools = tooled.tools.clone();
    let chats = tooled.chats.to_vec().clone();
    drop(tooled);
    drop(ollama);

    if tools.contains(&Tools::DuckDuckGo) {
        coordinator = coordinator.add_tool(DDGSearcher::new());
    } else if tools.contains(&Tools::Serper) {
        coordinator = coordinator.add_tool(SerperSearchTool {});
    } else if tools.contains(&Tools::Scraper) {
        coordinator = coordinator.add_tool(Scraper {});
    } else if tools.contains(&Tools::Finance) {
        coordinator = coordinator.add_tool(StockScraper::new());
    } else if tools.contains(&Tools::Calculator) {
        coordinator = coordinator.add_tool(Calculator {});
    }

    (coordinator, chats)
}

async fn get_message(
    coordinator: &mut Coordinator<Vec<ChatMessage>>,
    chats: Vec<ChatMessage>,
) -> Result<ChatMessage, String> {
    if let Ok(result) = coordinator.chat(chats).await {
        return Ok(result.message);
    }

    return Err("Failed to run ollama.".to_string());
}

pub async fn run_ollama_tools(
    tooled: Arc<TooledOptions>,
    options: ModelOptions,
    ollama: Arc<Mutex<Ollama>>,
) -> Result<ChatMessage, String> {
    let (mut coordinator, chats) = get_coordinator(tooled, options, ollama).await;
    get_message(&mut coordinator, chats).await
}

pub async fn delete_model(ollama: Arc<Mutex<Ollama>>, model: String) {
    let o = ollama.lock().await;
    let _ = o.delete_model(model).await;
}

pub fn run_ollama_stream(
    chats: Arc<Vec<ChatMessage>>,
    options: ModelOptions,
    ollama: Arc<Mutex<Ollama>>,
) -> impl Stream<Item = Result<ChatProgress, String>> {
    try_channel(1, |mut output| async move {
        let ollama = ollama.lock().await;
        let request = ChatMessageRequest::new(options.model().to_string(), chats.to_vec())
            .options(options.into());
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
    pub tools: Arc<Vec<Tools>>,
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
    options: ModelOptions,
    ollama: Arc<Mutex<Ollama>>,
) -> iced::Subscription<(ChatStreamId, Result<ChatProgress, String>)> {
    Subscription::run_with_id(
        id,
        run_ollama_stream(chats, options, ollama).map(move |progress| (id, progress)),
    )
}

impl ChatStream {
    pub fn new(app: &ChatApp, id: ChatStreamId, option: usize) -> Self {
        if let Some(chat) = app.chats.0.get(&id.0) {
            Self {
                state: State::Generating(ChatMessage::new(
                    ollama_rs::generation::chat::MessageRole::Assistant,
                    String::new(),
                )),
                chats: Arc::new(chat.get_chat_messages()),
                options: app.options.model_options()[option].clone(),
                tools: Arc::new(chat.tools.clone()),
            }
        } else {
            Self {
                state: State::Generating(ChatMessage::new(
                    ollama_rs::generation::chat::MessageRole::Assistant,
                    String::new(),
                )),
                chats: Arc::new(Vec::new()),
                options: app.options.model_options()[option].clone(),
                tools: Arc::new(Vec::new()),
            }
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
                self.options.clone(),
                app.logic.ollama.clone(),
            )
            .map(Message::Generating),
            _ => Subscription::none(),
        }
    }
}
