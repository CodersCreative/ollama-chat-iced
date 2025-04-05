use iced::{futures::{SinkExt, Stream, StreamExt, TryFutureExt}, stream::try_channel, Subscription, Task};
use ollama_rs::{
    generation::chat::{
        request::ChatMessageRequest, ChatMessage,
    }, models::pull::PullModelStatus, Ollama
};
use tokio::sync::Mutex;
use std::{sync::Arc, time::Instant, error::Error};

use crate::{options::ModelOptions, save::{chats::Chats, chat::Chat}, ChatApp, Message};




#[derive(Debug, Clone)]
pub enum ChatProgress {
    Generating(ChatMessage),
    Finished,
}

pub fn get_model() -> Ollama{
    return Ollama::default();
}


 
pub async fn run_ollama(chats: Vec<ChatMessage>, options : ModelOptions, ollama : Arc<Mutex<Ollama>>) -> Result<ChatMessage, String>{
    let o = ollama.lock().await;
 
    let request = ChatMessageRequest::new(options.1.clone(), chats.to_vec()).options(options.into());
    let result = o.send_chat_messages(request).await;
 
    if let Ok(result) = result{
        if result.message.is_none(){
            return Err("No Result".to_string());
        }
 
        return Ok(result.message.unwrap());
    }
 
    return Err("Failed to run ollama.".to_string());
}

pub async fn delete_model(ollama : Arc<Mutex<Ollama>>, model : String){
    let now = Instant::now();
    let o = ollama.lock().await;
    let _ = o.delete_model(model).await;
}


pub fn run_ollama_stream(chats: Arc<Vec<ChatMessage>>, options : ModelOptions, ollama : Arc<Mutex<Ollama>>) -> impl Stream<Item = Result<ChatProgress, String>>{
    try_channel(1, move |mut output| async move{
        let ollama = ollama.lock().await;
        let request = ChatMessageRequest::new(options.1.clone(), chats.to_vec()).options(options.into());
        let mut y = ollama.send_chat_messages_stream(request).await.map_err(|x|x.to_string())?;
        let _ = output.send(ChatProgress::Generating(ChatMessage { role: ollama_rs::generation::chat::MessageRole::Assistant, content: String::new(), images: None })).await;

        while let Some(Ok(response)) = y.next().await{
            if let Some(x) = response.message{
                let _ = output.send(ChatProgress::Generating(x)).await;
            }
        }

        let _ = output.send(ChatProgress::Finished).await;

        Ok(())
    })
}

#[derive(Debug)]
pub struct ChatStream {
    pub id: i32,
    pub state: State,
    pub chats: Arc<Vec<ChatMessage>>, 
    pub options : ModelOptions, 
}

#[derive(Debug)]
pub enum State {
    Generating(ChatMessage),
    Finished,
    Errored,
}

pub fn chat(id : i32, chats: Arc<Vec<ChatMessage>>, options : ModelOptions, ollama : Arc<Mutex<Ollama>>) -> iced::Subscription<(i32, Result<ChatProgress, String>)>{
    Subscription::run_with_id(id, run_ollama_stream(chats, options, ollama).map(move |progress| (id, progress)))
}

impl ChatStream{
    pub fn new(app : &ChatApp, id : i32, option : usize, chat : usize) -> Self {
        
        Self{
            id,
            state: State::Generating(ChatMessage::new(ollama_rs::generation::chat::MessageRole::Assistant, String::new())),
            chats : Arc::new(app.save.chats[chat].get_chat_messages()), 
            options : app.options.0[option].clone(), 
        }
    }

    pub fn progress(
        &mut self,
        new_progress: Result<ChatProgress, String>,
    ) {
        if let State::Generating(message) = &mut self.state {
            match new_progress {
                Ok(ChatProgress::Generating (mes)) => {
                    *message = mes;
                }
                Ok(ChatProgress::Finished) => {
                    self.state = State::Finished;
                }
                Err(e) => {
                    self.state = State::Errored;
                }
            }
        }
    }

    pub fn subscription(&self, app : &ChatApp) -> Subscription<Message> {
        match self.state {
            State::Generating (_) => {
                chat(self.id,self.chats.clone(), self.options.clone(), app.logic.ollama.clone()).map(Message::Generating)
            }
            _ => Subscription::none(),
        }
    }

}

