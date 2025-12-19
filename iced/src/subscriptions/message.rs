use crate::{DATA, get_client};
use iced::{
    Task,
    futures::StreamExt,
    task::{Straw, sipper},
};
use ochat_types::generation::text::{ChatQueryData, ChatResponse, ChatStreamResult};

#[derive(Debug, Clone)]
pub struct MessageGen {
    pub id: String,
    pub query: ChatQueryData,
    pub state: ChatStreamResult,
}

pub enum MessageGenUpdate {
    Generating(ChatStreamResult),
    Finished(Result<(), String>),
}

impl MessageGen {
    pub fn new(id: String, query: ChatQueryData) -> Self {
        Self {
            id,
            query,
            state: ChatStreamResult::Idle,
        }
    }

    pub fn start(&mut self) -> Task<MessageGenUpdate> {
        match self.state {
            ChatStreamResult::Err(_) | ChatStreamResult::Finished | ChatStreamResult::Idle => {
                let (task, _handle) = Task::sip(
                    gen_stream(self.query.clone()),
                    MessageGenUpdate::Generating,
                    MessageGenUpdate::Finished,
                )
                .abortable();

                self.state = ChatStreamResult::Generating(ChatResponse::default());

                task
            }
            _ => Task::none(),
        }
    }

    pub fn progress(&mut self, progress: ChatStreamResult) {
        self.state = progress;
    }
}

pub fn gen_stream(query: ChatQueryData) -> impl Straw<(), ChatStreamResult, String> {
    let url = DATA.read().unwrap().instance_url.clone().unwrap();

    sipper(async move |mut output| {
        let mut response = get_client()
            .get(&format!("{}/generation/text/stream/", url))
            .json(&query)
            .send()
            .await
            .unwrap()
            .bytes_stream();

        while let Some(status) = response.next().await {
            let _ = match serde_json::from_slice::<ChatStreamResult>(&status.unwrap()) {
                Ok(x) => {
                    let _ = output.send(x).await;
                }
                Err(e) => {
                    let _ = output.send(ChatStreamResult::Err(e.to_string())).await;
                }
            };
        }
        let _ = output.send(ChatStreamResult::Finished).await;
        Ok(())
    })
}
