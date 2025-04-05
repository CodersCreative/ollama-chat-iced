use iced::{ alignment::{Horizontal, Vertical}, event::listen, widget::{button, column, svg, text, Renderer}, Length, Task, Theme};
use kalosm_sound::{rodio::buffer::SamplesBuffer, MicInput};
use ollama_rs::generation::chat::ChatMessage;
use crate::{chat::run_ollama, panes::Panes, save::chat::Chat, sound::{get_audio, transcribe}, style, utils::get_path_assets, ChatApp, Message};
use iced::Element;
use std::{path::PathBuf, sync::Arc};

#[derive(Debug, Clone)]
pub struct Call {
    pub state : State,
    pub chats : Vec<Chat>,
    pub model : String,
}

impl Call{
    pub fn new(model : String) -> Self{
        Self{
            model,
            state : State::Idle,
            chats : Vec::new(),
        }
    }

    pub fn view(&self, app : &ChatApp) -> Element<Message>{
        let mdl = text(&self.model)
            .color(app.theme().palette().primary)
            .size(48)
            .width(Length::FillPortion(6))
            .align_y(Vertical::Center)
            .align_x(Horizontal::Center);

        let str = match self.state {
            State::Outputting => "Playing Audio",
            State::Listening => "Listening...",
            State::Generating => "Prompting AI",
            State::Idle => "Call Ended",
        };
        
        let txt = text(str)
            .color(app.theme().palette().primary)
            .size(24)
            .width(Length::FillPortion(6))
            .align_y(Vertical::Center)
            .align_x(Horizontal::Center);

        let btn = |file : &str| -> button::Button<Message, Theme, Renderer>{
            button(
                svg(svg::Handle::from_path(get_path_assets(file.to_string()))).style(style::svg::background).width(Length::Fixed(48.0)),
            )
            .style(style::button::rounded_primary)
            .width(Length::Fixed(64.0))
        };

        let call_btn =if self.state == State::Idle{
            btn(
                "call.svg"
            ).on_press(Message::Call(CallMessage::StartCall(app.call.model.clone())))
        }else{
            btn(
                "end_call.svg"
            ).on_press(Message::Call(CallMessage::EndCall))
        };


        return column![
            mdl,
            txt,
            call_btn,
        ].spacing(20).padding(40).align_x(Horizontal::Center).into()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum State{
    Generating,
    Listening,
    Outputting,
    Idle,
}

#[derive(Debug, Clone)]
pub enum CallMessage{
    StartCall(String),
    EndCall,
    ChangeModel(String),
    Listen,
    Convert(Option<SamplesBuffer<f32>>),
    Listened(Result<String, String>),
    Generated(Result<ChatMessage, String>)
}

impl CallMessage{
    pub fn handle(&self, app : &mut ChatApp) -> Task<Message>{
        match self{
            Self::StartCall(x) => {
                if let Some(_) = app.panes.focus{
                    app.call.model = x.clone();
                    let _ = app.options.get_create_model_options_index(x.clone());
                    Panes::new_window(app, app.panes.focus.unwrap().clone(), crate::panes::Pane::Call);
                    let mic = MicInput::default();
                    let stream = mic.stream();
                    
                    app.call.state = State::Listening;
                    return Task::perform(get_audio(stream), move |x| Message::Call(CallMessage::Convert(x)))
                }

                Task::none()
            },
            Self::EndCall => {
                app.call = Call::new(app.call.model.clone());
                Task::none()
            },
            Self::Listen => {
                if app.call.state.clone() != State::Idle{
                    let mic = MicInput::default();
                    let stream = mic.stream();
                    
                    app.call.state = State::Listening;
                    return Task::perform(get_audio(stream), move |x| Message::Call(CallMessage::Convert(x)));
                }

                Task::none()

            },
            Self::Convert(x) => {
                if app.call.state != State::Idle{
                    app.call.state = State::Generating;
                    return Task::perform(transcribe(x.clone()), move |x| Message::Call(CallMessage::Listened(x)));
                }
                Task::none()
            },
            Self::Listened(x) => {
                if app.call.state != State::Idle{
                    if let Ok(str) = x{
                        app.call.chats.push(Chat::new(&crate::save::chat::Role::User, str, Vec::new()));
                    }
                    let index = app.options.get_create_model_options_index(app.call.model.clone());

                    return Task::perform(run_ollama(app.call.chats.iter().map(|x| x.into()).collect::<Vec<ChatMessage>>(), app.options.0[index].clone(), app.logic.ollama.clone()), move |x| Message::Call(CallMessage::Generated(x)));
                }
                Task::none()
            },
            Self::Generated(x) => {
                if let Ok(str) = x{
                    if app.call.state.clone() != State::Idle{
                        app.call.chats.push(Chat::new(&crate::save::chat::Role::AI, &str.content, Vec::new()));
                        if let Err(_) = app.tts.say_auto(str.content.clone()){
                            app.tts.default_model = Some(natural_tts::Model::TTS);
                            let _ =app.tts.say_auto(str.content.clone()).unwrap();
                        }
                        println!("{}", str.content);

                    }else{
                        return Task::none();
                    }
                }

                let mic = MicInput::default();
                let stream = mic.stream();
                app.call.state = State::Listening;
                Task::perform(get_audio(stream), move |x| Message::Call(CallMessage::Convert(x)))
            },
            Self::ChangeModel(x) => {
                app.call.model = x.clone();
                let _ = app.options.get_create_model_options_index(x.clone());
                Task::none()
            },
        }
    }
}
