pub mod values;
pub mod convert;
use serde::{Deserialize, Serialize};
use crate::{style::{self}, utils::convert_image, Message};
use iced::{alignment::{Horizontal, Vertical}, widget::{self, button, column, container, row, text, text_input, toggler}, Element, Length};
use serde_json;
use std::time::SystemTime;
use std::{fs::File, io::Read};
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Options (pub Vec<GenOption>);

impl Default for Options {
    fn default() -> Self {
        Self::new()
    }
}

impl Options{
    pub fn save(&self, path : &str){
        let writer = File::create(path);

        if let Ok(writer) = writer {
            let _ = serde_json::to_writer(writer, &self);
        }
    }

    pub fn load(path : &str) -> Result<Self, String>{
        let reader = File::open(path);

        if let Ok(mut reader) = reader {
            let mut data = String::new();
            let _ = reader.read_to_string(&mut data).unwrap();

            let de_data = serde_json::from_str(&data);

            return match de_data {
                Ok(x) => Ok(x),
                Err(e) => Err(e.to_string()),
            };
        }

         return Err("Failed to open file".to_string());
    }

    pub fn view(&self) -> Element<Message>{
        column(self.0.iter().map(|x| {
            x.view()
        })).into()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum OptionKey{
    Mirostat,
    MirostatETA,
    MirostatTau,
    CtxWindow,
    NumGQA,
    GPULayers,
    NumThreads,
    RepeatN,
    RepeatPenalty,
    Temperature,
    Seed,
    StopSequence,
    TailFreeZ,
    NumberPredict,
    TopK,
    TopP,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GenOption{
    pub name: String,
    pub doc : String,
    pub shown: bool,
    pub key: OptionKey,
    num_type: NumType,
    pub temp: String,
    pub bool_value: bool,
    pub num_value: Option<(f32, f32)>,
    pub text_value: Option<(String, String)>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum NumType{
    Decimal,
    Whole
}

impl GenOption{

    fn new(name: &str, doc : &str, key : OptionKey, num_value : Option<(f32, f32)>, text_value : Option<(String, String)>) -> Self{
        Self{
            name: name.to_string(),
            doc: doc.to_string(),
            num_type: NumType::Whole,
            temp: num_value.unwrap().0.to_string(),
            key,
            shown: false,
            bool_value: false,
            num_value,
            text_value,
        }
    }

    fn with_type(&mut self, num_type: NumType){
        self.num_type = num_type;
    }

    pub fn view<'a>(&'a self) -> Element<Message>{
        if self.shown{
            let name = button(text(&self.name).center().size(16)).on_press(Message::ClickedOption(self.key.clone())).style(style::button::chosen_chat);
            let mut widgets : Vec<Element<Message>> = vec![
                row![
                    toggler(self.bool_value).label("Activated").on_toggle(|x| Message::ChangeOptionBool((x, self.key.clone()))).width(Length::FillPortion(3)),
                    button(
                        text("Reset").align_x(Horizontal::Center).align_y(Vertical::Center).width(Length::Fill).size(16)
                    )
                    .width(Length::FillPortion(2))
                    .style(style::button::not_chosen_chat)
                    .on_press(Message::ResetOption(self.key.clone())),

                ].spacing(10).into()
            ];

            if let Some(x) = self.num_value{
                widgets.push(text_input(&x.1.to_string(), &self.temp).on_input(|x| Message::ChangeOptionNum((x, self.key.clone()))).on_submit(Message::SubmitOptionNum(self.key.clone())).into());
            }
            let settings = container(column(widgets));

            container(column![
                name,
                settings,
            ]).style(style::container::code_darkened).padding(10).into()
        }else{
            button(text(&self.name).center().size(16)).on_press(Message::ClickedOption(self.key.clone())).style(style::button::not_chosen_chat).into()
        }

    }
}



