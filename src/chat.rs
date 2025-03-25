use ollama_rs::{
    generation::chat::{
        request::ChatMessageRequest, ChatMessage,
    }, Ollama
};
use std::time::Instant;
use tokio::sync::Mutex;
use std::sync::Arc;

use crate::save::chats::Chats;

pub fn get_model() -> Ollama{
    return Ollama::new_default_with_history(50);
}

pub async fn run_ollama(input : String, ollama : Arc<Mutex<Ollama>>, model : String) -> Result<String, String>{
    let user_message = ChatMessage::user(input.to_string());
    let now = Instant::now();
    let mut o = ollama.lock().await;
    let result = o.send_chat_messages_with_history(ChatMessageRequest::new(model.to_string(), vec![user_message]), "default".to_string()).await;
    println!("LLM Time: {}", now.elapsed().as_secs());

    if let Ok(result) = result{
        if result.message.is_none(){
            return Err("No Result".to_string());
        }
        let response = result.message.unwrap().content;
        return Ok(response.into());
    }
    return Err("Failed to run ollama.".to_string());
}

pub fn ollama_from_chat(ollama : Arc<Mutex<Ollama>>, chat : Chats){
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    tokio_runtime.block_on(ollama_chat_async(ollama, chat));
}

pub async fn ollama_chat_async(ollama : Arc<Mutex<Ollama>>, chat : Chats){
    let mut o = ollama.lock().await;
    
    *o = get_model();
    
    for c in chat.0{
        if c.name.as_str() == "AI"{
            o.add_assistant_response("default".to_string(), c.message.clone());
        }else{
            o.add_user_response("default".to_string(), c.message.clone());
        }
    }
}

pub async fn get_models(ollama : Arc<Mutex<Ollama>>) -> Vec<String>{
    let o = ollama.lock().await;
    return o.list_local_models().await.unwrap().iter().map(|x| x.name.clone()).collect::<Vec<String>>();
}

