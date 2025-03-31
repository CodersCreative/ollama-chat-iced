use std::str::FromStr;
use std::{error::Error, fs::File, io::Read};
use ollama_rs::models::pull::PullModelStatus;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, OwnedValue, Schema, STORED, TEXT};
use tantivy::index::Index;
use tantivy::{doc, DocAddress,IndexWriter, Score, TantivyDocument};
use url::Url;
use std::collections::HashMap;
use iced::{
    alignment::{Horizontal, Vertical},widget::{button, column, combo_box, container, keyed_column, row, scrollable, text, text_input, Renderer}, Element, Length, Task, Theme
};
use std::sync::Arc;
//use iced::task::{Straw, sipper};
//use crate::download::{ DownloadProgress};
use crate::utils::get_path_settings;
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


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
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
    //Delete(String),
    //Pulled(Result<PullModelStatus, String>),
    Search,
    Input(String)

}

impl ModelsMessage{
    pub fn handle(&self, models : Models, app : &mut ChatApp) -> Task<Message>{
        match self{
            Self::Expand(x) => {
                let index = Models::get_index(app, models.0);
                if models.1 != Some(x.clone()){
                    app.main_view.models[index].1 = Some(x.clone());
                }else{
                    app.main_view.models[index].1 = None;
                }
                Task::none()
            },

            //Self::Pulled(_) => {
            //    let models = app.logic.get_models();
            //    app.logic.models = models.clone();
            //    app.logic.combo_models = combo_box::State::new(models.clone());
            //    Task::none()
            //},
            Self::Input(x) => {
                let index = Models::get_index(app, models.0);
                app.main_view.models[index].2 = x.clone();
                Task::none()
            }
            Self::Search => {
                let index = Models::get_index(app, models.0);
                app.main_view.models[index].3 = app.model_info.search(models.2).unwrap();
                Task::none()
            }
        }
    }
}

impl ModelInfo{
    fn view<'a>(&'a self, app : &'a ChatApp, id : i32, expand : bool) -> Element<'a, Message>{
        let mut widgets : Vec<Element<Message>> = Vec::new();

        widgets.push(
            button(
                text(self.name.clone())
                .color(app.theme().palette().primary)
                .size(24)
                .width(Length::Fill)
                .align_y(Vertical::Center)
                .align_x(Horizontal::Left)
            ).style(style::button::transparent_back).on_press(Message::Models(ModelsMessage::Expand(self.name.clone()), id)).into()
        );

        widgets.push(text(&self.author)
        .color(app.theme().palette().danger)
        .size(20)
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
            widgets.push(
                button(text(&self.url).size(16)).style(style::button::chosen_chat).on_press(Message::URLClicked(Url::from_str(&self.url).unwrap())).into()
            );
            for tag in &self.tags{
                //let name = format!("{}:{}", self.name, tag[0]);
                widgets.push(button(
                    row![
                        text(tag[0].clone()).align_x(Horizontal::Center).align_y(Vertical::Center).width(Length::Fill).size(16),
                        text(tag[1].clone()).align_x(Horizontal::Center).align_y(Vertical::Center).width(Length::Fill).size(16)
                    ]
                )
                .style(style::button::not_chosen_chat)
                //.on_press(Message::Models(ModelsMessage::Pull(), id))
                .on_press(Message::Pull(format!("{}:{}", self.name, tag[0])))
                .width(Length::Fill).padding(10).into());
            } 

        }


        container(column(widgets).padding(10)).padding(5).style(style::container::side_bar).into()
    }

    fn search_format(&self) -> String{
        format!("{}, {:?}, {}, {:?}", self.name, self.tags, self.author, self.categories)
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

#[derive(Debug, Clone)]
pub struct SavedModels{
    pub models : Vec<ModelInfo>,
    pub descriptions : HashMap<String, String>,
    pub data : (Index, Vec<Field>, Schema),
}

impl Into<SaveableModels> for SavedModels{
    fn into(self) -> SaveableModels {
        SaveableModels{
            models : self.models,
            descriptions : self.descriptions,
        }
    }
}

impl Into<SavedModels> for SaveableModels{
    fn into(self) -> SavedModels {
        SavedModels{
            models : self.models.clone(),
            descriptions : self.descriptions.clone(),
            data : self.into_index().unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct SaveableModels{
    pub models : Vec<ModelInfo>,
    pub descriptions : HashMap<String, String>,
}

impl SaveableModels{
    pub fn save(&self, path : &str){
        let path = get_path_settings(path.to_string());
        let writer = File::create(path);

        if let Ok(writer) = writer {
            let _ = serde_json::to_writer(writer, &self);
        }
    }

    pub fn load(path : &str) -> Result<Self, String>{
        let path = get_path_settings(path.to_string());
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
    fn into_index<'a>(&'a self) -> Result<(Index, Vec<Field>, Schema), Box<dyn Error>>{
        let mut schema_builder = Schema::builder();
        let name = schema_builder.add_text_field("title", TEXT | STORED);
        let desc = schema_builder.add_text_field("desc", TEXT);
        let author = schema_builder.add_text_field("author", TEXT);
        let schema = schema_builder.build();

        let index = Index::create_in_ram(schema.clone());
        let mut index_writer: IndexWriter = index.writer(50_000_000)?;

        for model in &self.models{
            let d = self.descriptions.get(model.name.as_str()).unwrap().clone();
            index_writer.add_document(doc!(
                name => model.name.clone(),
                desc => d.as_str(),
                author => model.author.clone()
            ))?;
        }

        index_writer.commit()?;

        Ok((index, vec![name, desc, author], schema))
    }


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

        let models = Self{
            models,
            descriptions,
        };

        models.save(MODELS_PATH);

        Ok(models)
    }
}

impl SavedModels{
    pub fn save(&self, path : &str){
        let saveable : SaveableModels = self.clone().into();
        saveable.save(path)
    }

    pub fn init() -> Result<Self, Box<dyn Error>>{
        let saveable : SaveableModels = SaveableModels::init()?;
        Ok(saveable.into())
    }


    pub fn load(path : &str) -> Result<Self, String>{
        let saveable : SaveableModels = SaveableModels::load(path)?;
        Ok(saveable.into())
    }
    pub fn search<'a>(&'a self, input : String) -> Result<Vec<ModelInfo>, Box<dyn Error>>{
        if input.is_empty(){
            return Ok(self.models.clone());
        }

        let reader = self.data.0.reader()?;
        let searcher = reader.searcher();

        let query_parser = QueryParser::for_index(&self.data.0, self.data.1.clone());
        let query = query_parser.parse_query(&input)?;

        let top_docs: Vec<(Score, DocAddress)> = searcher.search(&query, &TopDocs::with_limit(10))?;
        let mut models = Vec::new();

        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc::<TantivyDocument>(doc_address)?;
            
            let model : String = match retrieved_doc.get_first(self.data.1[0].clone()).unwrap(){
                OwnedValue::Str(x) => x.clone(),
                _ => "".to_string(),
            };

            let model = self.models.iter().find(|x| x.name == model).unwrap();
            models.push(model.clone());
        }

        Ok(models)
    }
}

#[derive(Debug, Clone)]
pub struct Models(pub i32, pub Option<String>, pub String, pub Vec<ModelInfo>);

impl Models{
    pub fn new(app: &ChatApp) -> Self{
        Self(
            generate_id(),
            None,
            String::new(),
            app.model_info.models.clone()
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
            self.3
                .iter()
                .enumerate()
                .map(|(i, model)| {
                    let mut expand = false;
                    
                    if let Some(x) = &self.1{
                        expand = x == &model.name;
                    }

                    (
                        0,
                        model.view(app, self.0, expand)
                    )
                }),
        )
        .spacing(10)
        .into()
    }

    pub fn view<'a>(&'a self, app : &'a ChatApp,) -> Element<'a, Message>{
        let input = text_input::<Message, Theme, Renderer>("Enter your message", &self.2)
            .on_input(|x| Message::Models(ModelsMessage::Input(x), self.0))
            .on_submit(Message::Models(ModelsMessage::Search, self.0))
            //.on_submit(Message::Chats(ChatsMessage::Submit, self.id))
            .size(16)
            .style(style::text_input::input)
            .width(Length::Fill);
        
        container(column![
            input,
            scrollable::Scrollable::new(self.view_models(app)).width(Length::Fill)
        ]).width(Length::Fill)
        .height(Length::Fill)
        .padding(20).into()

    }
}

impl SavedModels{


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
