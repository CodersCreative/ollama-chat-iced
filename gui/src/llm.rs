use crate::{
    chats::chat::Role,
    common::Id,
    options::ModelOptions,
    providers::Provider,
    tools::{builtin::get_builtin_funcs, SavedTool, SavedToolFunc, ToolType},
    ChatApp, Message,
};
use async_openai::types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs};
use iced::{
    futures::{SinkExt, Stream, StreamExt},
    stream::try_channel,
    Subscription,
};

use serde::{Deserialize, Serialize};

#[cfg(feature = "python")]
use pyo3::{
    types::{IntoPyDict, PyModule},
    Bound, PyAny, Python,
};
#[cfg(feature = "python")]
use pythonize::pythonize;
use serde_json::Value;

#[cfg(feature = "python")]
use std::ffi::CString;

use std::{collections::HashMap, sync::Arc, usize};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum ChatProgress {
    Generating(ChatStreamResponse, Vec<ChatStreamResponse>),
    Finished,
}

#[derive(Debug, Clone, Default)]
pub struct ChatStreamResponse {
    pub role: Role,
    pub content: String,
}

pub fn run_llm_stream(
    chats: Arc<Vec<ChatCompletionRequestMessage>>,
    tools: Arc<Vec<SavedTool>>,
    options: ModelOptions,
    provider: Arc<Mutex<Provider>>,
) -> impl Stream<Item = Result<ChatProgress, String>> {
    try_channel(1, |mut output| async move {
        let provider = provider.lock().await;
        let tools = tools.to_vec();

        let request = Into::<CreateChatCompletionRequestArgs>::into(options)
            .messages(chats.to_vec())
            .build()
            .map_err(|e| e.to_string())?;

        let mut y = provider
            .client
            .chat()
            .create_stream(request)
            .await
            .map_err(|x| x.to_string())?;

        let _ = output
            .send(ChatProgress::Generating(
                ChatStreamResponse::default(),
                Vec::new(),
            ))
            .await;

        while let Some(Ok(response)) = y.next().await {
            let mut tool_responses: Vec<String> = Vec::new();
            let mut content = String::new();
            let mut tool_calls: Vec<(String, HashMap<String, Value>)> = Vec::new();

            for segment in response.choices {
                if let Some(text) = segment.delta.content {
                    content.push_str(&text);
                }

                if let Some(calls) = segment.delta.tool_calls {
                    tool_calls.append(
                        &mut calls
                            .into_iter()
                            .filter(|x| x.function.is_some())
                            .map(|x| {
                                let args = if let Some(args) = x.function.clone().unwrap().arguments
                                {
                                    if let Ok(x) = serde_json::from_str(&args) {
                                        x
                                    } else {
                                        return (String::from("err"), HashMap::new());
                                    }
                                } else {
                                    HashMap::new()
                                };

                                (x.function.unwrap().name.unwrap_or(String::new()), args)
                            })
                            .filter(|x| !x.0.is_empty() || x.0 == "err")
                            .collect(),
                    );
                }
            }

            if !tool_calls.is_empty() {
                #[cfg(feature = "python")]
                let mut pythons = HashMap::new();

                for call in tool_calls.iter() {
                    let call_name = &call.0;
                    let tool: Vec<(usize, SavedToolFunc)> = tools
                        .iter()
                        .enumerate()
                        .filter_map(|(i, x)| {
                            x.functions
                                .iter()
                                .find(|x| x.name.trim() == call_name.trim())
                                .map(|x| (i, x.clone()))
                        })
                        .collect();

                    if tool.is_empty() {
                        continue;
                    }

                    let tool = tool.first().unwrap();

                    let mut args = HashMap::new();
                    for param in tool.1.params.iter() {
                        if let Some(arg) = call.1.get(param.0.trim()) {
                            args.insert(param.0.trim().to_string(), arg.clone());
                        }
                    }

                    match tool.1.tool_type {
                        ToolType::Builtin => {
                            if let Some(func) = get_builtin_funcs().get(tool.1.name.trim()) {
                                tool_responses.push(func.run(args));
                            }
                        }
                        #[cfg(feature = "python")]
                        ToolType::Python => {
                            let module = if let Some(m) = pythons.get(&tool.0) {
                                m
                            } else {
                                let m = Python::attach(|py| {
                                    let code = CString::new(
                                        tools
                                            .get(tool.0)
                                            .unwrap()
                                            .python
                                            .clone()
                                            .unwrap()
                                            .to_string(),
                                    )
                                    .unwrap();
                                    let file = CString::new("tool.py".to_string()).unwrap();
                                    let module = CString::new("Tool".to_string()).unwrap();
                                    return PyModule::from_code(py, &code, &file, &module)
                                        .unwrap()
                                        .unbind();
                                });
                                pythons.insert(tool.0, m);
                                pythons.get(&tool.0).unwrap()
                            };

                            Python::attach(|py| {
                                if let Ok(tools_class) = module.getattr(py, "Tools") {
                                    let args: Vec<(String, Bound<'_, PyAny>)> = args
                                        .into_iter()
                                        .map(|x| (x.0, pythonize(py, &x.1).unwrap()))
                                        .collect();
                                    let result = tools_class
                                        .call0(py)
                                        .unwrap()
                                        .call(py, (), Some(&args.into_py_dict(py).unwrap()))
                                        .unwrap();
                                    tool_responses.push(result.extract::<String>(py).unwrap());
                                }
                            })
                        }
                        ToolType::Lua => {
                            todo!()
                        }
                    }
                }
            }

            let _ = output
                .send(ChatProgress::Generating(
                    ChatStreamResponse {
                        content,
                        role: Role::AI,
                    },
                    tool_responses
                        .into_iter()
                        .map(|x| ChatStreamResponse {
                            role: Role::Function,
                            content: x,
                        })
                        .collect(),
                ))
                .await;
        }

        let _ = output.send(ChatProgress::Finished).await;

        Ok(())
    })
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChatStreamId(pub Id, pub usize);

impl ChatStreamId {
    pub fn new(saved: Id, index: usize) -> Self {
        return Self(saved, index);
    }
}

#[derive(Debug)]
pub struct ChatStream {
    pub state: State,
    pub chats: Arc<Vec<ChatCompletionRequestMessage>>,
    pub provider: Arc<Mutex<Provider>>,
    pub options: ModelOptions,
    pub tools: Arc<Vec<SavedTool>>,
}

#[derive(Debug)]
pub enum State {
    Generating(ChatStreamResponse),
    Finished,
    Errored,
}

pub fn chat(
    id: ChatStreamId,
    chats: Arc<Vec<ChatCompletionRequestMessage>>,
    tools: Arc<Vec<SavedTool>>,
    options: ModelOptions,
    provider: Arc<Mutex<Provider>>,
) -> iced::Subscription<(ChatStreamId, Result<ChatProgress, String>)> {
    Subscription::run_with_id(
        id,
        run_llm_stream(chats, tools, options, provider).map(move |progress| (id, progress)),
    )
}

impl ChatStream {
    pub fn new(
        app: &ChatApp,
        chats: Arc<Vec<ChatCompletionRequestMessage>>,
        tools: Arc<Vec<SavedTool>>,
        model: usize,
        provider: Arc<Mutex<Provider>>,
    ) -> Self {
        Self {
            state: State::Generating(ChatStreamResponse::default()),
            chats: chats,
            options: app.options.model_options()[model].clone(),
            tools: tools,
            provider,
        }
    }

    pub fn progress(&mut self, new_progress: Result<ChatProgress, String>) {
        if let State::Generating(message) = &mut self.state {
            match new_progress {
                Ok(ChatProgress::Generating(mes, _)) => {
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

    pub fn subscription(&self, _app: &ChatApp, id: ChatStreamId) -> Subscription<Message> {
        match self.state {
            State::Generating(_) => chat(
                id,
                self.chats.clone(),
                self.tools.clone(),
                self.options.clone(),
                self.provider.clone(),
            )
            .map(Message::Generating),
            _ => Subscription::none(),
        }
    }
}
