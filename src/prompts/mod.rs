pub mod message;
pub mod view;

use crate::{
    common::Id,
    style,
    utils::{get_path_assets, get_path_settings},
    ChatApp, Message,
};
use derive_builder::Builder;
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{button, column, container, row, svg, text, text_editor, text_input},
    Element, Length, Padding, Renderer, Theme,
};
use message::PromptsMessage;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fs::File, io::Read};
use tantivy::{
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{Field, Schema, Value, STORED, TEXT},
    DocAddress, Index, IndexWriter, Score, TantivyDocument,
};
use view::Edit;

pub const PROMPTS_PATH: &str = "prompts.json";

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct Prompt {
    pub command: String,
    pub title: String,
    pub content: String,
}

impl From<&Edit> for Prompt {
    fn from(value: &Edit) -> Self {
        Self {
            content: value.content.text().to_string(),
            title: value.title.clone(),
            command: value.command.clone(),
        }
    }
}

impl Prompt {
    pub fn view<'a>(
        &'a self,
        app: &ChatApp,
        id: Id,
        expand: bool,
        edit: &'a Edit,
    ) -> Element<'a, Message> {
        let btn = |file: &str| -> button::Button<'a, Message, Theme, Renderer> {
            button(
                svg(svg::Handle::from_path(get_path_assets(file.to_string())))
                    .style(style::svg::primary)
                    .width(Length::Fixed(32.0)),
            )
            .style(style::button::chosen_chat)
            .width(Length::Fixed(48.0))
        };

        container(if !expand {
            column![
                button(
                    text(self.title.clone())
                        .color(app.theme().palette().primary)
                        .size(24)
                        .width(Length::Fill)
                        .align_y(Vertical::Center)
                        .align_x(Horizontal::Left)
                )
                .style(style::button::transparent_back)
                .padding(0)
                .on_press(Message::Prompts(
                    PromptsMessage::Expand(self.command.clone()),
                    id.clone(),
                )),
                text(&self.command)
                    .color(app.theme().palette().danger)
                    .size(20)
                    .width(Length::Fill)
                    .align_y(Vertical::Center)
                    .align_x(Horizontal::Left)
            ]
        } else {
            let title = text_input::<Message, Theme, Renderer>("Enter the title", &edit.title)
                .on_input(move |x| Message::Prompts(PromptsMessage::EditTitle(x), id.clone()))
                .on_submit(Message::Prompts(PromptsMessage::EditSave, id.clone()))
                .size(16)
                .style(style::text_input::input)
                .width(Length::Fill);

            let title = row![
                title,
                btn("save.svg").on_press(Message::Prompts(PromptsMessage::EditSave, id.clone(),)),
                btn("close.svg").on_press(Message::Prompts(
                    PromptsMessage::Expand(self.command.clone()),
                    id.clone(),
                )),
            ];
            let command =
                text_input::<Message, Theme, Renderer>("Enter the command", &edit.command)
                    .on_input(move |x| Message::Prompts(PromptsMessage::EditCommand(x), id.clone()))
                    .on_submit(Message::Prompts(PromptsMessage::EditSave, id.clone()))
                    .size(16)
                    .style(style::text_input::input)
                    .width(Length::Fill);
            let content = text_editor(&edit.content)
                .placeholder("Type the prompt content here...")
                .on_action(move |action| {
                    Message::Prompts(PromptsMessage::EditAction(action), id.clone())
                })
                .padding(Padding::from(20))
                .size(20)
                .style(style::text_editor::input)
                .key_binding(move |key_press| {
                    let modifiers = key_press.modifiers;

                    match text_editor::Binding::from_key_press(key_press) {
                        Some(text_editor::Binding::Enter) if !modifiers.shift() => {
                            Some(text_editor::Binding::Custom(Message::Prompts(
                                PromptsMessage::EditSave,
                                id.clone(),
                            )))
                        }
                        binding => binding,
                    }
                });
            column![title, command, content,]
        })
        .padding(5)
        .style(style::container::side_bar)
        .into()
    }
}

#[derive(Debug, Clone)]
pub struct SavedPrompts {
    pub prompts: HashMap<Id, Prompt>,
    pub data: (Index, Vec<Field>, Schema),
}

impl PartialEq for SavedPrompts {
    fn eq(&self, other: &Self) -> bool {
        self.prompts == other.prompts
    }
}

impl Default for SavedPrompts {
    fn default() -> Self {
        Self::new(HashMap::new())
    }
}

impl SavedPrompts {
    pub fn save(&self, path: &str) {
        let path = get_path_settings(path.to_string());
        let writer = File::create(path);

        if let Ok(writer) = writer {
            let _ = serde_json::to_writer_pretty(writer, &self.prompts);
        }
    }

    pub fn new(prompts: HashMap<Id, Prompt>) -> Self {
        Self {
            data: Self::into_index(&prompts).unwrap(),
            prompts,
        }
    }

    pub fn load(path: &str) -> Result<Self, String> {
        Ok(Self::new(Self::load_prompts(path)?))
    }

    async fn get_prompts_paths() -> Result<Vec<String>, String> {
        let files = rfd::AsyncFileDialog::new()
            .add_filter("Json", &["json"])
            .pick_files()
            .await;

        if let Some(files) = files {
            return Ok(files
                .iter()
                .map(|x| {
                    x.path()
                        .to_path_buf()
                        .into_os_string()
                        .into_string()
                        .unwrap()
                })
                .collect());
        }

        Err("Failed".to_string())
    }

    pub fn load_prompts(path: &str) -> Result<HashMap<Id, Prompt>, String> {
        let path = get_path_settings(path.to_string());
        let reader = File::open(path);

        if let Ok(mut reader) = reader {
            let mut data = String::new();
            let _ = reader
                .read_to_string(&mut data)
                .map_err(|e| e.to_string())?;

            let de_data = serde_json::from_str(&data);

            return match de_data {
                Ok(x) => Ok(x),
                Err(e) => Err(e.to_string()),
            };
        }

        return Err("Failed to open file".to_string());
    }

    pub fn import_new_prompts(&mut self, path: &str) -> Result<(), String> {
        let prompts = Self::load_from_file(path)?;
        for prompt in prompts {
            self.prompts.insert(Id::new(), prompt);
        }
        let _ = self.set_search_data().map_err(|e| e.to_string());
        Ok(())
    }

    fn load_from_file(path: &str) -> Result<Vec<Prompt>, String> {
        let reader = File::open(path);

        if let Ok(mut reader) = reader {
            let mut data = String::new();
            let _ = reader
                .read_to_string(&mut data)
                .map_err(|e| e.to_string())?;

            let de_data = serde_json::from_str(&data);

            return match de_data {
                Ok(x) => Ok(x),
                Err(e) => Err(e.to_string()),
            };
        }

        return Err("Failed to open file".to_string());
    }

    fn into_index(
        prompts: &HashMap<Id, Prompt>,
    ) -> Result<(Index, Vec<Field>, Schema), Box<dyn Error>> {
        let mut schema_builder = Schema::builder();
        let command = schema_builder.add_text_field("command", TEXT | STORED);
        let content = schema_builder.add_text_field("content", TEXT);
        let title = schema_builder.add_text_field("title", TEXT);
        let schema = schema_builder.build();

        let index = Index::create_in_ram(schema.clone());
        let mut index_writer: IndexWriter = index.writer(50_000_000)?;

        for (_, prompt) in prompts {
            index_writer.add_document(doc!(
                command => prompt.command.as_str(),
                content => prompt.content.as_str(),
                title => prompt.title.replace("-", " ").replace("_", " "),
            ))?;
        }

        index_writer.commit()?;

        Ok((index, vec![command, content, title], schema))
    }

    pub fn set_search_data(&mut self) -> Result<(), Box<dyn Error>> {
        self.data = Self::into_index(&self.prompts)?;
        Ok(())
    }

    pub fn search<'a>(&'a self, input: &'a str) -> Result<Vec<Prompt>, Box<dyn Error>> {
        if input.is_empty() || self.prompts.len() < 6 {
            return Ok(self.prompts.iter().map(|x| x.1.clone()).collect());
        }

        let reader = self.data.0.reader()?;
        let searcher = reader.searcher();

        let query_parser = QueryParser::for_index(&self.data.0, self.data.1.clone());
        let query = query_parser.parse_query(input)?;

        let top_docs: Vec<(Score, DocAddress)> =
            searcher.search(&query, &TopDocs::with_limit(5))?;

        let mut prompts = Vec::new();

        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc::<TantivyDocument>(doc_address)?;

            let command: String = match retrieved_doc
                .get_first(self.data.1[0].clone())
                .map(|x| x.as_str())
            {
                Some(Some(x)) => x.to_string(),
                _ => continue,
            };

            if let Some(prompt) = self.prompts.iter().find(|(_, x)| x.command == command) {
                prompts.push(prompt.1.clone());
            }
        }

        Ok(prompts)
    }
}
