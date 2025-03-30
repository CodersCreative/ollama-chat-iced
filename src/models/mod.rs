use std::{error::Error, fs::File, io::Read};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use iced::{
    alignment::{Horizontal, Vertical}, clipboard, widget::{column, combo_box, container, keyed_column, markdown, pane_grid, row, scrollable, text}, Element, Font, Length, Task, Theme
};

use crate::{style, utils::generate_id, ChatApp, Message};

const MODELS_PATH : &str = "models.json";

#[derive(Serialize, Deserialize, Debug)]
struct TempInfo {
    url: String,
    tags: Vec<Vec<String>>,
    author: String,
    categories: Vec<String>,
    languages: Vec<String>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ModelInfo {
    pub name: String,
    pub url: String,
    pub tags: Vec<Vec<String>>,
    pub author: String,
    pub categories: Vec<String>,
    pub languages: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ModelsMessage {
    Expand(String),
    Pull(String)
}

impl ModelInfo{
    fn view<'a>(&'a self, app : &'a ChatApp, expand : bool) -> Element<'a, Message>{
        let mut widgets : Vec<Element<Message>> = Vec::new();

        widgets.push(text(self.name.clone())
        .color(app.theme().palette().primary)
        .size(24)
        .width(Length::Fill)
        .align_y(Vertical::Center)
        .align_x(Horizontal::Left).into());

        if let Some(x) = app.model_info.descriptions.get(&self.name){
            widgets.push(text(x)
            .color(app.theme().palette().text)
            .size(16)
            .width(Length::Fill)
            .align_y(Vertical::Center)
            .align_x(Horizontal::Left).into());
        }

        if expand{
            widgets.push(text("hello").into());
            //let title = button(
            //    text(&self.title).align_x(Horizontal::Center).align_y(Vertical::Center).width(Length::Fill).size(20)
            //)
            //.style(style)
            //.on_press(Message::Chats(ChatsMessage::ChangeChat(self.id), app.panes.last_chat))
            //.width(Length::FillPortion(7)).padding(Padding::from(10));
        }


        container(column(widgets).padding(5)).style(style::container::chat_back).into()
    }
}

impl Into<ModelInfo> for TempInfo{
    fn into(self) -> ModelInfo {
        ModelInfo{
            name : String::new(),
            url : self.url,
            tags : self.tags,
            author : self.author,
            categories : self.categories,
            languages : self.languages,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SavedModels{
    pub models : Vec<ModelInfo>,
    pub descriptions : HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Models(pub i32, pub Option<String>);

impl Models{
    pub fn new() -> Self{
        Self(
            generate_id(),
            None,
        )
    }

    pub fn get_from_id<'a>(app: &'a ChatApp, id : i32) -> &'a Self{
        app.main_view.models.iter().find(|x| x.0 == id).unwrap()
    }

    pub fn get_index<'a>(app : &'a ChatApp, id : i32) -> usize{
        for i in 0..app.main_view.models.len(){
            if app.main_view.models[i].0 == id{
                return i
            }
        }
        0
    }
    pub fn view_models<'a>(&'a self, app : &'a ChatApp,) -> Element<'a, Message>{
        keyed_column(
            app.model_info.models
                .iter()
                .enumerate()
                .map(|(i, model)| {
                    let mut expand = false;
                    
                    if let Some(x) = &self.1{
                        expand = x == &model.name;
                    }

                    (
                        0,
                        model.view(app, expand)
                    )
                }),
        )
        .spacing(10)
        .into()
    }

    pub fn view<'a>(&'a self, app : &'a ChatApp,) -> Element<'a, Message>{
        container(scrollable::Scrollable::new(self.view_models(app)).width(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
    }
}

impl SavedModels{
    pub fn init() -> Result<Self, Box<dyn Error>>{
        if let Ok(x) = Self::load(MODELS_PATH){
            return Ok(x);
        }
        
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
        
        let resp = tokio_runtime.block_on(reqwest::get("https://raw.githubusercontent.com/Jeffser/Alpaca/main/src/available_models.json"))?;
        let content = tokio_runtime.block_on(resp.text())?;
        let data : HashMap<String, TempInfo> = serde_json::from_str(&content)?;

        let models : Vec<ModelInfo> = data.into_iter().map(|x| {
            let mut info : ModelInfo = x.1.into();
            info.name = x.0;
            info
        }).collect();


        let resp = tokio_runtime.block_on(reqwest::get("https://raw.githubusercontent.com/Jeffser/Alpaca/main/src/available_models_descriptions.py"))?;
        let content = tokio_runtime.block_on(resp.text())?;
        let descriptions = extract_model_description(&content)?;
        println!("{:#?}", descriptions);

        let models = Self{
            models,
            descriptions,
        };

        models.save(MODELS_PATH);

        Ok(models)
    }
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
}


fn extract_model_description(python_code: &str) -> Result<HashMap<String, String>, String> {
    let mut result = HashMap::new();

    // Remove the "descriptions = " part and the curly braces
    let code_without_prefix = python_code
        .trim()
        .trim_start_matches("descriptions = {")
        .trim_end_matches("}");

    // Split the string into key-value pairs
    let pairs: Vec<&str> = code_without_prefix.split(",\n").map(|s| s.trim()).collect();

    let key_regex = Regex::new(r"'([^']+)'").unwrap();
    let value_regex = Regex::new(r#"_\("([^"]+)"\)"#).unwrap();

    for pair in pairs {
        if pair.is_empty() {
            continue;
        }

        let key_capture = key_regex.captures(pair);
        let value_capture = value_regex.captures(pair);

        if let (Some(key_caps), Some(value_caps)) = (key_capture, value_capture) {
            let key = key_caps.get(1).map_or("", |m| m.as_str()).trim().to_string();
            let value = value_caps.get(1).map_or("", |m| m.as_str()).trim().to_string();
            //println!("{:?}", value);
            result.insert(key, value);
        } else {
            //c
            return Err(format!("Failed to parse pair: {}", pair));
        }
    }

    Ok(result)
}
