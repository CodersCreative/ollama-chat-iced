use iced::{
    Element, Task,
    widget::{column, container, pick_list, rule, scrollable},
};
use iced_selection::text;
use ochat_common::data::RequestType;
use ochat_types::{
    chats::messages::Role,
    generation::{
        text::{ChatQueryData, ChatQueryMessage, ChatResponse},
        tts::{TtsQueryData, TtsResponse},
    },
    settings::SettingsProvider,
};
use std::rc::Rc;

use crate::{
    Application, DATA, Message,
    font::{BODY_SIZE, SUB_HEADING_SIZE},
    pages::home::panes::view::HomePaneViewMessage,
    style,
    subscriptions::{SubMessage, recorder::RecorderState},
};

#[derive(Debug, Clone, Default)]
pub struct CallView {
    pub model: Option<SettingsProvider>,
    pub stt_model: Option<SettingsProvider>,
    pub tts_model: Option<SettingsProvider>,
    pub history: Vec<ChatQueryMessage>,
    pub tools: Vec<String>,
    pub state: CallState,
    pub ref_count: u8,
    pub stop: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum CallState {
    #[default]
    Idle,
    Recording(u32),
    GeneratingText,
    GeneratingTts,
    Playing,
}

#[derive(Debug, Clone)]
pub enum CallViewMessage {
    StartRecording,
    StartGenerating(String),
    ResponseGenerated(ChatResponse),
    StartPlaying(TtsResponse),
    UpdateSttModel(SettingsProvider),
    UpdateTtsModel(SettingsProvider),
    UpdateModel(SettingsProvider),
    Stop,
}

impl CallViewMessage {
    pub fn handle(self, app: &mut Application) -> Task<Message> {
        match self {
            Self::StartRecording => {
                #[cfg(not(feature = "sound"))]
                {
                    return Task::done(Message::Err(String::from(
                        "ERROR : ochat build does not support calling...",
                    )));
                }

                if !app
                    .cache
                    .server_features
                    .contains(&ochat_types::ServerFeatures::Sound)
                {
                    return Task::done(Message::Err(String::from(
                        "ERROR : Server does not support calling...",
                    )));
                }

                let sub_id = app.subscriptions.counter;

                let provider = {
                    let view = app.view_data.home.call.as_mut().unwrap();
                    if view.stop {
                        view.state = CallState::Idle;
                        view.stop = false;
                        return Task::none();
                    } else {
                        view.state = CallState::Recording(sub_id);
                    }
                    view.stt_model.clone()
                };

                Task::done(Message::Subscription(SubMessage::Record(
                    provider,
                    crate::subscriptions::recorder::RecorderFinish(Rc::new(move |app, txt| {
                        if let Some(txt) = txt {
                            Task::done(Message::HomePaneView(HomePaneViewMessage::Call(
                                CallViewMessage::StartGenerating(txt),
                            )))
                        } else {
                            if let Some(view) = app.view_data.home.call.as_mut() {
                                if let CallState::Recording(_) = view.state {
                                    view.state = CallState::Idle;
                                    Task::done(Message::Err(String::from(
                                        "Unable to transcribe audio",
                                    )))
                                } else {
                                    Task::none()
                                }
                            } else {
                                Task::done(Message::Err(String::from("Unable to find call pane")))
                            }
                        }
                    })),
                )))
            }
            Self::StartGenerating(txt) => {
                let query = {
                    let view = app.view_data.home.call.as_mut().unwrap();
                    if view.stop {
                        view.state = CallState::Idle;
                        view.stop = false;
                        return Task::none();
                    }

                    view.state = CallState::GeneratingText;
                    view.history.push(ChatQueryMessage {
                        text: txt,
                        files: Vec::new(),
                        role: Role::User,
                    });

                    ChatQueryData {
                        messages: view.history.clone(),
                        force_disable_tools: false,
                        tools: view.tools.clone(),
                        provider: view.model.as_ref().unwrap().provider.clone(),
                        model: view.model.as_ref().unwrap().model.clone(),
                    }
                };

                Task::future(async move {
                    let req = DATA.read().unwrap().to_request();
                    match req
                        .make_request::<ChatResponse, ChatQueryData>(
                            "generation/text/run/",
                            &query,
                            RequestType::Get,
                        )
                        .await
                    {
                        Ok(message) => Message::HomePaneView(HomePaneViewMessage::Call(
                            CallViewMessage::ResponseGenerated(message),
                        )),
                        Err(e) => Message::Err(e),
                    }
                })
            }
            Self::ResponseGenerated(response) => {
                let query = {
                    let view = app.view_data.home.call.as_mut().unwrap();
                    if view.stop {
                        view.state = CallState::Idle;
                        view.stop = false;
                        return Task::none();
                    } else {
                        view.state = CallState::GeneratingTts;
                    }

                    view.history.push(ChatQueryMessage {
                        text: response.content.clone(),
                        files: Vec::new(),
                        role: Role::AI,
                    });

                    TtsQueryData {
                        text: response.content,
                        model: view.tts_model.clone(),
                    }
                };

                Task::future(async move {
                    let req = DATA.read().unwrap().to_request();
                    match req
                        .make_request::<TtsResponse, TtsQueryData>(
                            "generation/tts/run/",
                            &query,
                            RequestType::Get,
                        )
                        .await
                    {
                        Ok(data) => Message::HomePaneView(HomePaneViewMessage::Call(
                            CallViewMessage::StartPlaying(data),
                        )),
                        Err(e) => Message::Err(e),
                    }
                })
            }
            Self::StartPlaying(response) => {
                {
                    let view = app.view_data.home.call.as_mut().unwrap();
                    if view.stop {
                        view.state = CallState::Idle;
                        view.stop = false;
                        return Task::none();
                    } else {
                        view.state = CallState::Playing
                    }
                }

                Task::done(Message::Subscription(SubMessage::Play(
                    response,
                    crate::subscriptions::player::PlayerFinish(Rc::new(move |app| {
                        if app.view_data.home.call.is_some() {
                            Task::done(Message::HomePaneView(HomePaneViewMessage::Call(
                                CallViewMessage::StartRecording,
                            )))
                        } else {
                            Task::done(Message::Err(String::from("Unable to find call pane")))
                        }
                    })),
                )))
            }
            Self::UpdateSttModel(x) => {
                app.view_data.home.call.as_mut().unwrap().stt_model = Some(x);
                Task::none()
            }
            Self::UpdateTtsModel(x) => {
                app.view_data.home.call.as_mut().unwrap().tts_model = Some(x);
                Task::none()
            }
            Self::Stop => {
                app.view_data.home.call.as_mut().unwrap().stop = true;
                Task::none()
            }
            Self::UpdateModel(x) => {
                app.view_data.home.call.as_mut().unwrap().model = Some(x);
                Task::none()
            }
        }
    }
}

impl CallView {
    pub fn view<'a>(&'a self, app: &'a Application) -> Element<'a, Message> {
        let sub_heading = |txt: &'static str| text(txt).size(BODY_SIZE).style(style::text::primary);

        let mut model_column = column([]).spacing(5);

        if let Ok(x) = DATA.read() {
            if !x.models.is_empty() {
                let default_model = pick_list(
                    x.models.clone(),
                    app.cache.client_settings.default_provider.clone(),
                    move |x| {
                        Message::HomePaneView(HomePaneViewMessage::Call(
                            CallViewMessage::UpdateModel(x),
                        ))
                    },
                )
                .style(style::pick_list::main)
                .menu_style(style::menu::main);

                model_column = model_column.push(sub_heading("Default Model"));
                model_column = model_column.push(default_model);

                let mut added_sound = false;

                #[cfg(feature = "sound")]
                {
                    if app
                        .cache
                        .server_features
                        .contains(&ochat_types::ServerFeatures::Sound)
                    {
                        let stt_model = pick_list(
                            x.stt_models.clone(),
                            app.cache.client_settings.stt_provider.clone(),
                            move |x| {
                                Message::HomePaneView(HomePaneViewMessage::Call(
                                    CallViewMessage::UpdateSttModel(x),
                                ))
                            },
                        )
                        .style(style::pick_list::main)
                        .menu_style(style::menu::main);

                        model_column = model_column.push(sub_heading("STT Model"));
                        model_column = model_column.push(stt_model);

                        let tts_model = pick_list(
                            x.tts_models.clone(),
                            app.cache.client_settings.tts_provider.clone(),
                            move |x| {
                                Message::HomePaneView(HomePaneViewMessage::Call(
                                    CallViewMessage::UpdateTtsModel(x),
                                ))
                            },
                        )
                        .style(style::pick_list::main)
                        .menu_style(style::menu::main);

                        model_column = model_column.push(sub_heading("TTS Model"));
                        model_column = model_column.push(tts_model);
                        added_sound = true;
                    }
                }

                if !added_sound {
                    model_column = model_column.push(text("Either the server or this build of the iced application do not support audio features... if a call is attempted it may result in an error.").size(SUB_HEADING_SIZE).style(style::text::danger))
                }
            }
        }
        container(
            scrollable::Scrollable::new(
                column![
                    model_column,
                    rule::horizontal(1).style(style::rule::translucent::text),
                    text(match &self.state {
                        CallState::Idle => "Start the call",
                        CallState::Recording(x) =>
                            match app.subscriptions.recordings.get(x).map(|x| &x.state) {
                                Some(RecorderState::Idle) | None => "Starting to Record...",
                                Some(RecorderState::Recording) => "Recording...",
                                Some(RecorderState::Generating) => "Generating Text...",
                                Some(RecorderState::Err(_)) => "Unknown Error Occured",
                                _ => "Finishing up...",
                            },
                        CallState::GeneratingText => "Generating LLM Response",
                        CallState::GeneratingTts if self.history.len() <= 1 => "Converting LLM Response to Audio... ( May need to download new parler-tts model data which could take > 1hr )",
                        CallState::GeneratingTts  => "Converting LLM Response to Audio...",
                        CallState::Playing => "Playing Audio",
                    })
                    .style(style::text::text)
                    .size(SUB_HEADING_SIZE),
                    if self.state == CallState::Idle {
                        style::svg_button::primary("call.svg", 48).on_press(Message::HomePaneView(
                            HomePaneViewMessage::Call(CallViewMessage::StartRecording),
                        ))
                    } else {
                        style::svg_button::danger("end_call.svg", 48).on_press(
                            Message::HomePaneView(HomePaneViewMessage::Call(CallViewMessage::Stop)),
                        )
                    }
                ]
                .spacing(10),
            )
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::default(),
            )),
        )
        .into()
    }
}
