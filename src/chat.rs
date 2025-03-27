use ollama_rs::{
    generation::{chat::{
        request::ChatMessageRequest, ChatMessage,
    }, options::GenerationOptions}, Ollama
};
use tokio::sync::Mutex;
use std::{sync::Arc, time::Instant};

use crate::options::Options;

pub fn get_model() -> Ollama{
    return Ollama::new_default_with_history(50);
}

pub async fn run_ollama(chats: Arc<Vec<ChatMessage>>, options : Options, ollama : Arc<Mutex<Ollama>>, model : String) -> Result<ChatMessage, String>{
    let now = Instant::now();
    let o = ollama.lock().await;
    let request = ChatMessageRequest::new(model, chats.to_vec()).options(options.into());
    let result = o.send_chat_messages(request).await;
    println!("LLM Time: {}", now.elapsed().as_secs());

    if let Ok(result) = result{
        if result.message.is_none(){
            return Err("No Result".to_string());
        }
        //let response = result.message.unwrap();
        return Ok(result.message.unwrap());
    }
    return Err("Failed to run ollama.".to_string());
}


pub async fn get_models(ollama : Arc<Mutex<Ollama>>) -> Vec<String>{
    let o = ollama.lock().await;
    return o.list_local_models().await.unwrap().iter().map(|x| x.name.clone()).collect::<Vec<String>>();
}

