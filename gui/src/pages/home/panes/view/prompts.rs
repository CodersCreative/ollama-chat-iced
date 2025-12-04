use crate::{
    Application, DATA, InputMessage, Message,
    font::{BODY_SIZE, HEADER_SIZE, SUB_HEADING_SIZE},
    pages::home::panes::{data::PromptsData, view::HomePaneViewMessage},
    style,
    utils::{get_path_assets, load_from_file},
};
use iced::{
    Element, Length, Task,
    alignment::Vertical,
    widget::{
        column, container, horizontal_rule, horizontal_space, row,
        scrollable::{self, Scrollbar},
        svg, text, text_editor, text_input,
    },
};
use ochat_types::prompts::{Prompt, PromptData, PromptDataBuilder};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PromptsView {
    pub search: String,
    pub expanded: Vec<String>,
    pub editing: HashMap<String, PromptEdit>,
    pub prompts: PromptsData,
}

#[derive(Debug)]
pub struct PromptEdit {
    pub title: String,
    pub command: String,
    pub content: text_editor::Content,
}

impl From<Prompt> for PromptEdit {
    fn from(value: Prompt) -> Self {
        Self {
            title: value.title,
            command: value.command,
            content: text_editor::Content::with_text(&value.content),
        }
    }
}

impl Clone for PromptEdit {
    fn clone(&self) -> Self {
        Self {
            title: self.title.clone(),
            command: self.command.clone(),
            content: text_editor::Content::with_text(&self.content.text()),
        }
    }
}

impl Default for PromptsView {
    fn default() -> Self {
        Self {
            search: String::default(),
            expanded: Vec::new(),
            editing: HashMap::new(),
            prompts: PromptsData::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PromptsViewMessage {
    Search(InputMessage),
    UpdateTitle(String, String),
    UpdateCommand(String, String),
    UpdateContent(String, text_editor::Action),
    SavePrompt(String),
    SetPrompts(PromptsData),
    Expand(String),
    Edit(String),
    Delete(String),
    Add,
    Upload,
    Uploaded(Result<Vec<String>, String>),
    AddPrompt(Prompt),
}

impl PromptsViewMessage {
    pub fn handle(self, app: &mut Application, id: u32) -> Task<Message> {
        match self {
            Self::Search(InputMessage::Update(x)) => {
                let view = app.get_prompts_view(&id).unwrap();

                if x.is_empty() {
                    view.prompts.0.clear();
                }

                view.search = x;
                Task::none()
            }
            Self::Search(_) => {
                let search = app.get_prompts_view(&id).unwrap().search.clone();
                Task::future(async move {
                    Message::HomePaneView(HomePaneViewMessage::Prompts(
                        id,
                        PromptsViewMessage::SetPrompts(
                            PromptsData::get_prompts(Some(search)).await,
                        ),
                    ))
                })
            }
            Self::SetPrompts(x) => {
                app.get_prompts_view(&id).unwrap().prompts = x;
                Task::none()
            }
            Self::UpdateTitle(prompt_id, title) => {
                app.get_prompts_view(&id)
                    .unwrap()
                    .editing
                    .get_mut(&prompt_id)
                    .unwrap()
                    .title = title;
                Task::none()
            }
            Self::SavePrompt(prompt_id) => {
                let mut prompt = app
                    .cache
                    .home_shared
                    .prompts
                    .0
                    .iter()
                    .find(|x| x.id.key().to_string() == prompt_id)
                    .unwrap()
                    .clone();

                let edit = app
                    .get_prompts_view(&id)
                    .unwrap()
                    .editing
                    .get(&prompt_id)
                    .unwrap()
                    .clone();

                prompt.title = edit.title;
                prompt.command = edit.command;
                prompt.content = edit.content.text();

                app.cache
                    .home_shared
                    .prompts
                    .0
                    .iter_mut()
                    .filter(|x| x.id.key().to_string() == prompt_id)
                    .for_each(|x| *x = prompt.clone());

                app.view_data
                    .home
                    .prompts
                    .iter_mut()
                    .filter(|x| !x.1.prompts.0.is_empty())
                    .for_each(|x| {
                        x.1.prompts
                            .0
                            .iter_mut()
                            .filter(|x| x.id.key().to_string() == prompt_id)
                            .for_each(|x| *x = prompt.clone())
                    });

                Task::future(async move {
                    let req = DATA.read().unwrap().to_request();
                    let prompt: PromptData = prompt.into();

                    let _ = req
                        .make_request::<Prompt, PromptData>(
                            &format!("prompt/{}", prompt_id),
                            &prompt,
                            crate::data::RequestType::Put,
                        )
                        .await;
                    Message::None
                })
            }
            Self::UpdateCommand(prompt_id, command) => {
                app.get_prompts_view(&id)
                    .unwrap()
                    .editing
                    .get_mut(&prompt_id)
                    .unwrap()
                    .command = command;
                Task::none()
            }
            Self::UpdateContent(prompt_id, action) => {
                app.get_prompts_view(&id)
                    .unwrap()
                    .editing
                    .get_mut(&prompt_id)
                    .unwrap()
                    .content
                    .perform(action);
                Task::none()
            }
            Self::Expand(x) => {
                let view = app.get_prompts_view(&id).unwrap();
                if view.expanded.contains(&x) {
                    view.expanded.retain(|y| y != &x);
                } else {
                    view.expanded.push(x);
                }

                Task::none()
            }
            Self::Edit(x) => {
                let prompt = app
                    .cache
                    .home_shared
                    .prompts
                    .0
                    .iter()
                    .find(|y| &y.id.key().to_string() == &x)
                    .unwrap()
                    .clone();
                let view = app.get_prompts_view(&id).unwrap();

                if view.editing.contains_key(&x) {
                    view.editing.remove(&x);
                } else {
                    view.editing.insert(x, prompt.into());
                }

                Task::none()
            }
            Self::Upload => Task::perform(Self::get_prompts_paths(), move |x| {
                Message::HomePaneView(HomePaneViewMessage::Prompts(
                    id,
                    PromptsViewMessage::Uploaded(x),
                ))
            }),
            Self::Uploaded(x) => {
                let mut tasks = Vec::new();
                if let Ok(paths) = x {
                    for path in paths {
                        let prompts: Vec<PromptData> = match load_from_file(&path) {
                            Ok(x) => vec![x],
                            _ => match load_from_file(&path) {
                                Ok(x) => x,
                                Err(e) => {
                                    eprintln!("{:?}", e);
                                    continue;
                                }
                            },
                        };

                        for prompt in prompts.into_iter() {
                            tasks.push(Task::future(async move {
                                let req = DATA.read().unwrap().to_request();

                                match req
                                    .make_request::<Prompt, PromptData>(
                                        "prompt/",
                                        &prompt,
                                        crate::data::RequestType::Post,
                                    )
                                    .await
                                {
                                    Ok(x) => Message::HomePaneView(HomePaneViewMessage::Prompts(
                                        id,
                                        PromptsViewMessage::AddPrompt(x),
                                    )),
                                    _ => Message::None,
                                }
                            }));
                        }
                    }
                }

                Task::batch(tasks)
            }
            Self::AddPrompt(x) => {
                app.cache.home_shared.prompts.0.push(x);
                Task::none()
            }
            Self::Add => Task::future(async move {
                let req = DATA.read().unwrap().to_request();

                match req
                    .make_request::<Prompt, PromptData>(
                        "prompt/",
                        &PromptDataBuilder::default()
                            .title(String::from("New Prompt"))
                            .command(String::new())
                            .content(String::new())
                            .build()
                            .unwrap(),
                        crate::data::RequestType::Post,
                    )
                    .await
                {
                    Ok(x) => Message::HomePaneView(HomePaneViewMessage::Prompts(
                        id,
                        PromptsViewMessage::AddPrompt(x),
                    )),
                    _ => Message::None,
                }
            }),
            Self::Delete(x) => {
                app.cache
                    .home_shared
                    .prompts
                    .0
                    .retain(|y| y.id.key().to_string() != x);

                Task::future(async move {
                    let req = DATA.read().unwrap().to_request();

                    let _ = req
                        .make_request::<Prompt, ()>(
                            &format!("prompt/{}", x),
                            &(),
                            crate::data::RequestType::Delete,
                        )
                        .await;

                    Message::None
                })
            }
        }
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
}

impl PromptsView {
    pub fn view_prompt<'a>(
        id: u32,
        prompt: &'a Prompt,
        expanded: bool,
        edit_data: Option<&'a PromptEdit>,
    ) -> Element<'a, Message> {
        let stats: Option<Element<'a, Message>> = if prompt.user.is_some() {
            let mut inner_col = column![
                text("Stats (Open-WebUI)")
                    .size(BODY_SIZE)
                    .style(style::text::text),
            ]
            .spacing(5);

            let stat = |icon: &str, val: String| -> Element<'a, Message> {
                let icon = svg(svg::Handle::from_path(get_path_assets(icon.to_string())))
                    .style(style::svg::text)
                    .width(BODY_SIZE)
                    .height(BODY_SIZE);

                let val = text(val).size(BODY_SIZE);

                row![icon, val].spacing(10).align_y(Vertical::Center).into()
            };

            inner_col = inner_col.push(stat("account.svg", prompt.user.clone().unwrap().username));

            inner_col = inner_col.push(
                row![
                    stat(
                        "downloads.svg",
                        prompt.downloads.unwrap_or_default().to_string()
                    ),
                    stat(
                        "thumbs_up.svg",
                        prompt.upvotes.unwrap_or_default().to_string()
                    ),
                    stat(
                        "thumbs_down.svg",
                        prompt.downvotes.unwrap_or_default().to_string()
                    ),
                ]
                .spacing(20)
                .align_y(Vertical::Center),
            );

            Some(
                container(inner_col)
                    .style(style::container::chat_back)
                    .padding(10)
                    .into(),
            )
        } else {
            None
        };

        let edit = style::svg_button::primary(
            if edit_data.is_some() {
                "close.svg"
            } else {
                "edit.svg"
            },
            HEADER_SIZE,
        )
        .on_press(Message::HomePaneView(HomePaneViewMessage::Prompts(
            id,
            PromptsViewMessage::Edit(prompt.id.key().to_string()),
        )));

        let expand = style::svg_button::primary(
            if expanded {
                "arrow_drop_up.svg"
            } else {
                "arrow_drop_down.svg"
            },
            HEADER_SIZE,
        )
        .on_press(Message::HomePaneView(HomePaneViewMessage::Prompts(
            id,
            PromptsViewMessage::Expand(prompt.id.key().to_string()),
        )));

        let title = text(&prompt.title)
            .size(HEADER_SIZE)
            .style(style::text::primary);

        let command = text(&prompt.command)
            .size(BODY_SIZE)
            .style(style::text::text);

        let col = if let Some(edit_data) = edit_data {
            let sub_heading =
                |txt: &'static str| text(txt).size(BODY_SIZE).style(style::text::primary);

            let title = text_input("Enter a title...", &edit_data.title)
                .size(HEADER_SIZE)
                .style(style::text_input::input)
                .on_input(move |x| {
                    Message::HomePaneView(HomePaneViewMessage::Prompts(
                        id,
                        PromptsViewMessage::UpdateTitle(prompt.id.key().to_string(), x),
                    ))
                });

            let command = text_input("Enter a command...", &edit_data.command)
                .size(BODY_SIZE)
                .style(style::text_input::input)
                .on_input(move |x| {
                    Message::HomePaneView(HomePaneViewMessage::Prompts(
                        id,
                        PromptsViewMessage::UpdateCommand(prompt.id.key().to_string(), x),
                    ))
                });

            let content = text_editor(&edit_data.content)
                .on_action(move |x| {
                    Message::HomePaneView(HomePaneViewMessage::Prompts(
                        id,
                        PromptsViewMessage::UpdateContent(prompt.id.key().to_string(), x),
                    ))
                })
                .size(BODY_SIZE)
                .style(style::text_editor::input);

            let delete = style::svg_button::danger("delete.svg", HEADER_SIZE).on_press(
                Message::HomePaneView(HomePaneViewMessage::Prompts(
                    id,
                    PromptsViewMessage::Delete(prompt.id.key().to_string()),
                )),
            );

            let save = style::svg_button::primary("save.svg", HEADER_SIZE).on_press(
                Message::HomePaneView(HomePaneViewMessage::Prompts(
                    id,
                    PromptsViewMessage::SavePrompt(prompt.id.key().to_string()),
                )),
            );

            let col = column![
                row![delete, title, horizontal_space(), edit, save].align_y(Vertical::Center),
                horizontal_rule(1).style(style::rule::translucent::primary),
                sub_heading("command"),
                command,
                sub_heading("content"),
                content,
            ]
            .spacing(10);

            col
        } else if expanded {
            let mut col = column![
                row![title, horizontal_space(), edit, expand].align_y(Vertical::Center),
                horizontal_rule(1).style(style::rule::translucent::primary),
                command,
                text(&prompt.content)
                    .size(BODY_SIZE)
                    .style(style::text::translucent::text)
            ]
            .spacing(10);

            if let Some(stats) = stats {
                col = col.push(stats);
            }

            col
        } else {
            column![
                row![title, horizontal_space(), edit, expand].align_y(Vertical::Center),
                horizontal_rule(1).style(style::rule::translucent::primary),
                command
            ]
            .spacing(10)
        };

        container(col).into()
    }

    pub fn view<'a>(&'a self, app: &'a Application, id: u32) -> Element<'a, Message> {
        let search = style::svg_input::primary(
            Some(String::from("search.svg")),
            text_input("Search prompts...", &self.search)
                .on_input(move |x| {
                    Message::HomePaneView(HomePaneViewMessage::Prompts(
                        id,
                        PromptsViewMessage::Search(InputMessage::Update(x)),
                    ))
                })
                .on_submit(Message::HomePaneView(HomePaneViewMessage::Prompts(
                    id,
                    PromptsViewMessage::Search(InputMessage::Submit),
                ))),
            SUB_HEADING_SIZE,
        );

        let add = style::svg_button::primary("add.svg", SUB_HEADING_SIZE).on_press(
            Message::HomePaneView(HomePaneViewMessage::Prompts(id, PromptsViewMessage::Add)),
        );

        let upload = style::svg_button::primary("upload.svg", SUB_HEADING_SIZE).on_press(
            Message::HomePaneView(HomePaneViewMessage::Prompts(id, PromptsViewMessage::Upload)),
        );

        let prompts = scrollable::Scrollable::new(
            column(
                if self.search.is_empty() || self.prompts.0.is_empty() {
                    &app.cache.home_shared.prompts.0
                } else {
                    &self.prompts.0
                }
                .iter()
                .map(|x| {
                    Self::view_prompt(
                        id.clone(),
                        x,
                        self.expanded.contains(&x.id.key().to_string()),
                        self.editing.get(&x.id.key().to_string()),
                    )
                }),
            )
            .spacing(10),
        )
        .direction(scrollable::Direction::Vertical(Scrollbar::new()))
        .width(Length::Fill)
        .height(Length::Fill);

        container(column![row![search, add, upload], prompts].spacing(10)).into()
    }
}
