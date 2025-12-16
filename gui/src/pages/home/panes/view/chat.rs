use crate::{
    Application, DATA, Message, PopUp,
    data::{
        RequestType,
        start::{self, Section},
    },
    font::{BODY_SIZE, HEADER_SIZE, SMALL_SIZE, SUB_HEADING_SIZE, get_bold_font},
    pages::home::panes::{
        data::{MessageMk, PromptsData, ViewFile},
        view::HomePaneViewMessage,
    },
    style,
    subscriptions::SubMessage,
    utils::{self, get_path_assets},
};
use base64_stream::base64::{DecodeSliceError, Engine, decode, prelude::BASE64_STANDARD};
use iced::{
    Element, Length, Padding, Task, Theme,
    alignment::{Horizontal, Vertical},
    clipboard,
    widget::{
        button, center, column, container, image, lazy, markdown, mouse_area, pick_list, row, rule,
        scrollable, space, stack, svg, text, text_editor,
    },
};
use ochat_types::{
    chats::{
        Chat,
        messages::{MessageData, MessageDataBuilder, ModelData, Role},
    },
    files::{B64File, B64FileData, B64FileDataBuilder, DBFile, FileType},
    generation::text::{ChatQueryData, ChatQueryMessage},
    settings::SettingsProvider,
};
use std::{collections::HashMap, path::Path, sync::Arc};

const IMAGE_FORMATS: [&str; 15] = [
    "bmp", "dds", "ff", "gif", "hdr", "ico", "jpeg", "jpg", "exr", "png", "pnm", "qoi", "tga",
    "tiff", "webp",
];

#[derive(Debug, Clone)]
pub struct ChatsView {
    pub input: text_editor::Content,
    pub files: Vec<ViewFile>,
    pub models: Vec<SettingsProvider>,
    pub edits: HashMap<String, text_editor::Content>,
    pub expanded_messages: Vec<String>,
    pub prompts: PromptsData,
    pub selected_prompt: Option<String>,
    pub messages: Vec<String>,
    pub path: Vec<i8>,
    pub chat: Chat,
    pub start: usize,
}

#[derive(Debug, Clone)]
pub enum ChatsViewMessage {
    SetPrompts(PromptsData),
    ApplyPrompt(Option<String>),
    ChangePrompt(text_editor::Motion),
    SetInput(text_editor::Content),
    InputAction(text_editor::Action),
    SubmitInput,
    CancelGenerating,
    SelectFiles,
    FilesSelected(Vec<String>),
    FileUploaded(ViewFile),
    RemoveFile(usize),
    UserMessageUploaded(MessageMk),
    AIMessageUploaded(MessageMk, Option<ChatQueryData>),
    Regenerate(String),
    Branch(String),
    Expand(String),
    Edit(String),
    EditAction(String, text_editor::Action),
    SubmitEdit(String),
    AddModel,
    ChangeModel(usize, SettingsProvider),
    RemoveModel(usize),
    ChangeStart(usize),
}

impl ChatsViewMessage {
    pub fn handle(self, app: &mut Application, id: u32) -> Task<Message> {
        match self {
            Self::SetInput(x) => {
                let view = app.get_chats_view(&id).unwrap();
                view.input = x;
                view.selected_prompt = None;
                view.prompts.0.clear();
                Task::none()
            }
            Self::InputAction(action) => {
                let view = app.get_chats_view(&id).unwrap();
                view.prompts.0.clear();
                view.selected_prompt = None;
                view.input.perform(action);
                let search = get_command_input(&view.input.text())
                    .unwrap_or_default()
                    .to_string();

                if search.is_empty() {
                    Task::none()
                } else {
                    Task::future(async move {
                        match PromptsData::get(Some(search)).await {
                            Ok(x) => Message::HomePaneView(HomePaneViewMessage::Chats(
                                id,
                                ChatsViewMessage::SetPrompts(x),
                            )),
                            Err(e) => Message::Err(e),
                        }
                    })
                }
            }
            Self::SetPrompts(x) => {
                app.get_chats_view(&id).unwrap().prompts = x;
                Task::none()
            }
            Self::ChangePrompt(motion) => {
                let view = app.get_chats_view(&id).unwrap();
                let index = if let Some(s) = view.selected_prompt.clone() {
                    view.prompts
                        .0
                        .iter()
                        .position(|x| x.id.key().to_string() == s)
                } else {
                    None
                };

                let index: Option<usize> = match motion {
                    text_editor::Motion::Up => {
                        if let Some(selected) = index {
                            if selected > 0 {
                                Some(selected.clone() - 1)
                            } else if selected >= view.prompts.0.len() {
                                Some(0)
                            } else {
                                Some(view.prompts.0.len() - 1)
                            }
                        } else if view.prompts.0.len() > 0 {
                            Some(view.prompts.0.len() - 1)
                        } else {
                            None
                        }
                    }
                    _ => {
                        if let Some(selected) = index {
                            if selected < (view.prompts.0.len() - 2) {
                                Some(selected.clone() + 1)
                            } else {
                                Some(0)
                            }
                        } else if view.prompts.0.len() > 0 {
                            Some(0)
                        } else {
                            None
                        }
                    }
                };

                if let Some(index) = index {
                    view.selected_prompt =
                        view.prompts.0.get(index).map(|x| x.id.key().to_string());
                } else {
                    view.selected_prompt = None;
                }

                Task::none()
            }
            Self::ApplyPrompt(x) => {
                let view = app.get_chats_view(&id).unwrap();
                let prompt = if let Some(x) = x {
                    x
                } else {
                    view.selected_prompt.clone().unwrap()
                };

                let prompt = view
                    .prompts
                    .0
                    .iter()
                    .find(|y| y.id.key().to_string() == prompt)
                    .map(|x| x.clone())
                    .unwrap();

                clipboard::read_primary().map(move |clip| {
                    Message::HomePaneView(HomePaneViewMessage::Chats(
                        id,
                        ChatsViewMessage::SetInput(text_editor::Content::with_text(
                            &prompt
                                .content
                                .replace("{{CLIPBOARD}}", &clip.unwrap_or_default()),
                        )),
                    ))
                })
            }
            Self::SelectFiles => Task::perform(Self::get_file_paths(), move |x| match x {
                Ok(x) => Message::HomePaneView(HomePaneViewMessage::Chats(
                    id,
                    ChatsViewMessage::FilesSelected(x),
                )),
                Err(e) => Message::Err(e),
            }),
            Self::FilesSelected(paths) => {
                let mut files = Vec::new();
                for path in paths {
                    let file = Path::new(&path);
                    let file = match utils::convert_image(&file) {
                        Ok(x) => B64FileDataBuilder::default()
                            .file_type(FileType::Image)
                            .filename(path.rsplit_once("/").unwrap().1.to_string())
                            .b64data(x)
                            .build()
                            .unwrap(),
                        Err(e) => {
                            app.add_popup(PopUp::Err(e.to_string()));
                            continue;
                        }
                    };

                    files.push(file);
                }

                Task::batch(files.into_iter().map(|file| {
                    Task::future(async move {
                        let req = DATA.read().unwrap().to_request();
                        match req
                            .make_request::<DBFile, B64FileData>(&"file/", &file, RequestType::Post)
                            .await
                        {
                            Ok(x) => Message::HomePaneView(HomePaneViewMessage::Chats(
                                id,
                                ChatsViewMessage::FileUploaded(ViewFile {
                                    filename: file.filename,
                                    data: BASE64_STANDARD.decode(file.b64data).unwrap(),
                                    file_type: file.file_type,
                                    id: x.id.key().to_string().trim().to_string(),
                                }),
                            )),
                            Err(e) => Message::Err(e),
                        }
                    })
                }))
            }
            Self::FileUploaded(x) => {
                app.get_chats_view(&id).unwrap().files.push(x);
                Task::none()
            }
            Self::RemoveFile(index) => {
                let _ = app.get_chats_view(&id).unwrap().files.remove(index);
                Task::none()
            }
            Self::CancelGenerating => {
                let messages = app.get_chats_view(&id).unwrap().messages.clone();
                let ids: Vec<u32> = app
                    .subscriptions
                    .message_gens
                    .iter()
                    .filter(|x| messages.contains(&x.1.id))
                    .map(|x| x.0.clone())
                    .collect();

                Task::batch(
                    ids.into_iter()
                        .map(|x| Task::done(Message::Subscription(SubMessage::StopGenMessage(x)))),
                )
            }
            Self::SubmitInput => {
                let (user_message, parent, chat_id) = {
                    let view = app.get_chats_view(&id).unwrap();
                    let ret = (
                        MessageDataBuilder::default()
                            .content(view.input.text())
                            .files(view.files.iter().map(|x| x.id.trim().to_string()).collect())
                            .role(Role::User)
                            .build()
                            .unwrap(),
                        view.messages.last().map(|x| x.to_string()),
                        view.chat.id.key().to_string(),
                    );
                    view.input = text_editor::Content::new();
                    view.files.clear();
                    ret
                };

                Task::future(async move {
                    let req = DATA.read().unwrap().to_request();
                    match req
                        .make_request::<ochat_types::chats::messages::Message, MessageData>(
                            &if let Some(parent) = &parent {
                                format!("message/parent/{}", parent)
                            } else {
                                "message/".to_string()
                            },
                            &user_message,
                            RequestType::Post,
                        )
                        .await
                    {
                        Ok(x) => {
                            let msg = if parent.is_none() {
                                match req
                                    .make_request::<Chat, ()>(
                                        &format!(
                                            "chat/{}/root/{}",
                                            chat_id,
                                            x.id.key().to_string()
                                        ),
                                        &(),
                                        RequestType::Put,
                                    )
                                    .await
                                {
                                    Ok(_) => Message::None,
                                    Err(e) => Message::Err(e),
                                }
                            } else {
                                Message::None
                            };

                            Message::Batch(vec![
                                Message::HomePaneView(HomePaneViewMessage::Chats(
                                    id,
                                    ChatsViewMessage::UserMessageUploaded(MessageMk::get(x).await),
                                )),
                                msg,
                            ])
                        }
                        Err(e) => Message::Err(e),
                    }
                })
            }
            Self::UserMessageUploaded(user_message) => {
                if let Some(x) = app
                    .get_chats_view(&id)
                    .unwrap()
                    .messages
                    .last()
                    .map(|x| x.clone())
                {
                    for view in app
                        .view_data
                        .home
                        .chats
                        .iter_mut()
                        .filter(|y| y.1.messages.contains(&x))
                    {
                        view.1.messages.push(user_message.base.id.key().to_string());
                    }
                } else {
                    // TODO Do a filter to find all empty chats that use the same chat id.
                    app.get_chats_view(&id)
                        .unwrap()
                        .messages
                        .push(user_message.base.id.key().to_string());
                }

                app.cache
                    .home_shared
                    .messages
                    .0
                    .insert(user_message.base.id.key().to_string(), user_message.clone());
                let messages = app.get_chats_view(&id).unwrap().messages.clone();

                let messages: Vec<ChatQueryMessage> = messages
                    .iter()
                    .map(|x| {
                        app.cache
                            .home_shared
                            .messages
                            .0
                            .get(x)
                            .unwrap()
                            .base
                            .clone()
                            .into()
                    })
                    .collect();

                let req = DATA.read().unwrap().to_request();

                Task::batch(app.get_chats_view(&id).unwrap().models.clone().into_iter().map(|x| {
                    let user_message = user_message.base.id.key().to_string();
                    let messages = messages.clone();
                    let req = req.clone();
                    let id = id.clone();
                    Task::future(async move {
                        let message = MessageDataBuilder::default()
                            .content(String::new())
                            .role(Role::AI)
                            .model(Some(ModelData {provider : x.provider.trim().to_string(), model: x.model.clone()}))
                            .reason(Some(ochat_types::chats::relationships::Reason::Model))
                            .build()
                            .unwrap();
                        match req
                            .make_request::<ochat_types::chats::messages::Message, MessageData>(
                                &format!("message/parent/{}", user_message),
                                &message,
                                RequestType::Post,
                            )
                            .await
                        {
                            Ok(message) => Message::HomePaneView(HomePaneViewMessage::Chats(
                                id,
                                ChatsViewMessage::AIMessageUploaded(MessageMk::get(message).await, Some(ChatQueryData { provider: x.provider.trim().to_string(), model: x.model, messages }))
                            )),
                            Err(e) => Message::Err(e)
                        }
                    })
                }))
            }
            Self::AIMessageUploaded(message, query) => {
                if let Some(x) = app
                    .get_chats_view(&id)
                    .unwrap()
                    .messages
                    .last()
                    .map(|x| x.clone())
                {
                    for view in app
                        .view_data
                        .home
                        .chats
                        .iter_mut()
                        .filter(|y| y.1.messages.contains(&x))
                    {
                        view.1.messages.push(message.base.id.key().to_string());
                    }
                }

                let id = message.base.id.key().to_string();
                app.cache
                    .home_shared
                    .messages
                    .0
                    .insert(id.clone(), message.clone());

                if let Some(query) = query {
                    Task::done(Message::Subscription(SubMessage::GenMessage(id, query)))
                } else {
                    Task::none()
                }
            }
            Self::Expand(x) => {
                let view = app.get_chats_view(&id).unwrap();

                if view.expanded_messages.contains(&x) {
                    let _ = view.expanded_messages.retain(|y| y != &x);
                } else {
                    let _ = view.expanded_messages.push(x);
                }

                Task::none()
            }
            Self::Edit(x) => {
                let text = app
                    .cache
                    .home_shared
                    .messages
                    .0
                    .get(&x)
                    .unwrap()
                    .base
                    .content
                    .clone();

                let view = app.get_chats_view(&id).unwrap();

                if view.edits.contains_key(&x) {
                    let _ = view.edits.remove(&x);
                } else {
                    let _ = view.edits.insert(x, text_editor::Content::with_text(&text));
                }

                Task::none()
            }
            Self::EditAction(x, action) => {
                if let Some(msg) = app.get_chats_view(&id).unwrap().edits.get_mut(&x) {
                    msg.perform(action);
                }
                Task::none()
            }
            Self::SubmitEdit(message_id) => {
                let edit = app
                    .get_chats_view(&id)
                    .unwrap()
                    .edits
                    .remove(&message_id)
                    .unwrap();
                let message = match app.cache.home_shared.messages.0.get_mut(&message_id) {
                    Some(message) => {
                        message.base.content = edit.text();
                        message.content = markdown::Content::parse(&edit.text());
                        message.base.clone()
                    }
                    _ => return Task::done(Message::Err(String::from("Unable to find message."))),
                };

                Task::future(async move {
                    let req = DATA.read().unwrap().to_request();
                    match req
                        .make_request::<ochat_types::chats::messages::Message, MessageData>(
                            &format!("message/{}", message_id),
                            &message.into(),
                            RequestType::Put,
                        )
                        .await
                    {
                        Ok(_) => Message::None,
                        Err(e) => Message::Err(e),
                    }
                })
            }
            Self::Regenerate(message_id) => {
                let messages = app.get_chats_view(&id).unwrap().messages.clone();

                let index = messages.iter().position(|x| x == &message_id).unwrap();

                let parent = messages[index - 1].clone();
                let messages: Vec<ChatQueryMessage> = messages[0..index]
                    .iter()
                    .map(|x| {
                        app.cache
                            .home_shared
                            .messages
                            .0
                            .get(x)
                            .unwrap()
                            .base
                            .clone()
                            .into()
                    })
                    .collect();

                let model = app
                    .get_chats_view(&id)
                    .unwrap()
                    .models
                    .first()
                    .unwrap()
                    .clone();

                Task::future(async move {
                    let message = MessageDataBuilder::default()
                        .content(String::new())
                        .role(Role::AI)
                        .build()
                        .unwrap();

                    let req = DATA.read().unwrap().to_request();
                    match req
                        .make_request::<ochat_types::chats::messages::Message, MessageData>(
                            &format!("message/parent/{}", parent),
                            &message,
                            RequestType::Post,
                        )
                        .await
                    {
                        Ok(message) => Message::HomePaneView(HomePaneViewMessage::Chats(
                            id,
                            ChatsViewMessage::AIMessageUploaded(
                                MessageMk::get(message).await,
                                Some(ChatQueryData {
                                    provider: model.provider,
                                    model: model.model,
                                    messages,
                                }),
                            ),
                        )),
                        Err(e) => Message::Err(e),
                    }
                })
            }
            Self::Branch(_) => {
                // TODO
                Task::none()
            }
            Self::AddModel => {
                if let Some(model) = match app.cache.client_settings.default_provider.clone() {
                    Some(x) => Some(x),
                    _ => DATA.read().unwrap().models.first().map(|x| x.clone()),
                } {
                    app.get_chats_view(&id).unwrap().models.push(model);
                }
                Task::none()
            }
            Self::ChangeModel(index, model) => {
                *app.get_chats_view(&id)
                    .unwrap()
                    .models
                    .get_mut(index)
                    .unwrap() = model;
                Task::none()
            }
            Self::RemoveModel(index) => {
                let view = app.get_chats_view(&id).unwrap();

                if view.models.len() > 1 {
                    let _ = view.models.remove(index);
                }
                Task::none()
            }
            Self::ChangeStart(index) => {
                app.get_chats_view(&id).unwrap().start = index;
                Task::none()
            }
        }
    }

    async fn get_file_paths() -> Result<Vec<String>, String> {
        let files = rfd::AsyncFileDialog::new()
            .add_filter("Image", &IMAGE_FORMATS)
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

pub fn get_command_input(input: &str) -> Option<&str> {
    if let Some(split) = input.split_whitespace().last() {
        if split.contains("/") {
            return Some(split.trim_start_matches("/"));
        }
    }

    None
}

impl ChatsView {
    pub fn view_message<'a>(
        id: u32,
        theme: &Theme,
        message: &'a MessageMk,
        edit: Option<&'a text_editor::Content>,
        expanded: bool,
    ) -> Element<'a, Message> {
        let header = container(
            row(if edit.is_some() {
                let widgets: Vec<Element<'a, Message>> = vec![
                    text(message.base.role.to_string())
                        .size(BODY_SIZE)
                        .font(get_bold_font())
                        .into(),
                    if let Some(model) = &message.base.model {
                        text(format!("({})", model.model)).size(BODY_SIZE).into()
                    } else {
                        space().into()
                    },
                    space::horizontal().into(),
                    style::svg_button::danger("close.svg", BODY_SIZE)
                        .on_press(Message::HomePaneView(HomePaneViewMessage::Chats(
                            id,
                            ChatsViewMessage::Edit(message.base.id.key().to_string()),
                        )))
                        .into(),
                    style::svg_button::primary("save.svg", BODY_SIZE)
                        .on_press(Message::HomePaneView(HomePaneViewMessage::Chats(
                            id,
                            ChatsViewMessage::SubmitEdit(message.base.id.key().to_string()),
                        )))
                        .into(),
                ];

                widgets
            } else {
                let mut widgets: Vec<Element<'a, Message>> = vec![
                    text(message.base.role.to_string())
                        .font(get_bold_font())
                        .size(BODY_SIZE)
                        .into(),
                    if let Some(model) = &message.base.model {
                        text(format!("({})", model.model)).size(BODY_SIZE).into()
                    } else {
                        space().into()
                    },
                    space::horizontal().into(),
                    style::svg_button::text("edit.svg", BODY_SIZE)
                        .on_press(Message::HomePaneView(HomePaneViewMessage::Chats(
                            id,
                            ChatsViewMessage::Edit(message.base.id.key().to_string()),
                        )))
                        .into(),
                    style::svg_button::text("restart.svg", BODY_SIZE)
                        .on_press(Message::HomePaneView(HomePaneViewMessage::Chats(
                            id,
                            ChatsViewMessage::Regenerate(message.base.id.key().to_string()),
                        )))
                        .into(),
                    style::svg_button::text("branch.svg", BODY_SIZE)
                        .on_press(Message::HomePaneView(HomePaneViewMessage::Chats(
                            id,
                            ChatsViewMessage::Branch(message.base.id.key().to_string()),
                        )))
                        .into(),
                    style::svg_button::text("copy.svg", BODY_SIZE)
                        .on_press(Message::SaveToClipboard(message.base.content.clone()))
                        .into(),
                ];

                if message.can_change {
                    widgets.push(style::svg_button::text("back_arrow.svg", BODY_SIZE).into());

                    widgets.push(style::svg_button::text("forward_arrow.svg", BODY_SIZE).into());
                }

                widgets
            })
            .spacing(10)
            .align_y(Vertical::Center),
        )
        .padding(5)
        .style(if message.base.role == Role::User {
            style::container::chat
        } else {
            style::container::chat_ai
        })
        .width(Length::Fill);

        let images = lazy(&message.files, |files| {
            container(
                scrollable::Scrollable::new(
                    row(files
                        .iter()
                        .filter(|x| x.file_type == FileType::Image)
                        .map(|x| {
                            button(
                                image(image::Handle::from_bytes(x.data.clone()))
                                    .height(Length::Fixed(200.0)),
                            )
                            .style(style::button::transparent_back_white_text)
                            .into()
                        }))
                    .align_y(Vertical::Center)
                    .spacing(10),
                )
                .direction(scrollable::Direction::Horizontal(
                    scrollable::Scrollbar::new(),
                )),
            )
            .padding(Padding::from([0, 20]))
            .style(style::container::bottom_input_back)
        });

        let content: Element<'a, Message> = if let Some(edit) = edit {
            text_editor(&edit)
                .on_action(move |x| {
                    Message::HomePaneView(HomePaneViewMessage::Chats(
                        id.clone(),
                        ChatsViewMessage::EditAction(message.base.id.key().to_string(), x),
                    ))
                })
                .into()
        } else {
            markdown::view_with(
                message.content.items(),
                style::markdown::main(theme),
                &style::markdown::CustomViewer,
            )
        };

        let mut col = column![header, images, content,].spacing(5);

        if message.thinking.is_some() {
            let thinking: Element<'a, Message> = if expanded {
                mouse_area(
                    container(markdown::view_with(
                        message.thinking.as_ref().unwrap().items(),
                        style::markdown::main(theme),
                        &style::markdown::CustomViewer,
                    ))
                    .padding(10)
                    .style(style::container::back),
                )
                .on_press(Message::HomePaneView(HomePaneViewMessage::Chats(
                    id,
                    ChatsViewMessage::Expand(message.base.id.key().to_string()),
                )))
                .into()
            } else {
                button(
                    row![
                        svg(svg::Handle::from_path(get_path_assets("ai.svg"))).width(BODY_SIZE),
                        text("Thinking").size(BODY_SIZE)
                    ]
                    .align_y(Vertical::Center)
                    .spacing(10),
                )
                .padding(10)
                .style(style::button::chosen_chat)
                .on_press(Message::HomePaneView(HomePaneViewMessage::Chats(
                    id,
                    ChatsViewMessage::Expand(message.base.id.key().to_string()),
                )))
                .width(Length::Fill)
                .into()
            };

            col = col.push(thinking);
        }

        container(col)
            .padding(10)
            .style(style::container::chat_back)
            .into()
    }

    pub fn view<'a>(&'a self, app: &'a Application, id: u32) -> Element<'a, Message> {
        let is_generating = app
            .subscriptions
            .message_gens
            .iter()
            .find(|x| self.messages.contains(&x.1.id))
            .is_some();

        let input: Element<Message> = if !is_generating {
            text_editor(&self.input)
                .placeholder("Type your message here...")
                .on_action(move |action| {
                    Message::HomePaneView(HomePaneViewMessage::Chats(
                        id,
                        ChatsViewMessage::InputAction(action),
                    ))
                })
                .padding(Padding::from(20))
                .size(20)
                .style(style::text_editor::input)
                .key_binding(move |key_press| {
                    let modifiers = key_press.modifiers;

                    let is_command = !self.prompts.0.is_empty();

                    Some(text_editor::Binding::Custom(Message::HomePaneView(
                        HomePaneViewMessage::Chats(
                            id,
                            match text_editor::Binding::from_key_press(key_press) {
                                Some(text_editor::Binding::Enter)
                                    if !modifiers.shift() && is_command =>
                                {
                                    ChatsViewMessage::ApplyPrompt(None)
                                }
                                Some(text_editor::Binding::Move(text_editor::Motion::Up))
                                    if !modifiers.shift() && is_command =>
                                {
                                    ChatsViewMessage::ChangePrompt(text_editor::Motion::Up)
                                }
                                Some(text_editor::Binding::Move(text_editor::Motion::Down))
                                    if !modifiers.shift() && is_command =>
                                {
                                    ChatsViewMessage::ChangePrompt(text_editor::Motion::Down)
                                }
                                Some(text_editor::Binding::Enter) if !modifiers.shift() => {
                                    ChatsViewMessage::SubmitInput
                                }
                                binding => return binding,
                            },
                        ),
                    )))
                })
                .into()
        } else {
            container(
                text("Awaiting Response...")
                    .color(app.theme().palette().primary)
                    .size(20),
            )
            .padding(20)
            .width(Length::Fill)
            .style(container::transparent)
            .into()
        };

        let btn = |file: &'static str| style::svg_button::primary(file, 48);

        let btn_small = |file: &'static str| style::svg_button::primary(file, BODY_SIZE);

        let upload = btn("upload.svg").on_press(Message::HomePaneView(HomePaneViewMessage::Chats(
            id,
            ChatsViewMessage::SelectFiles,
        )));

        let submit: Element<Message> = match is_generating {
            true => btn("close.svg")
                .on_press(Message::HomePaneView(HomePaneViewMessage::Chats(
                    id,
                    ChatsViewMessage::CancelGenerating,
                )))
                .into(),
            false => btn("send.svg")
                .on_press(Message::HomePaneView(HomePaneViewMessage::Chats(
                    id,
                    ChatsViewMessage::SubmitInput,
                )))
                .into(),
        };

        let bottom = container(
            row![upload, input, submit]
                .align_y(Vertical::Center)
                .spacing(5),
        )
        .max_height(350);

        let files = lazy(&self.files, move |files| {
            container(
                scrollable::Scrollable::new(
                    row(files.iter().enumerate().map(|(i, x)| {
                        button(image(image::Handle::from_bytes(x.data.clone())).height(100))
                            .style(style::button::transparent_back_white_text)
                            .on_press(Message::HomePaneView(HomePaneViewMessage::Chats(
                                id,
                                ChatsViewMessage::RemoveFile(i),
                            )))
                            .into()
                    }))
                    .align_y(Vertical::Center)
                    .spacing(5),
                )
                .direction(scrollable::Direction::Horizontal(
                    scrollable::Scrollbar::new(),
                )),
            )
            .style(style::container::bottom_input_back)
        });

        let models = container(
            row![
                scrollable::Scrollable::new(
                    row(self
                        .models
                        .clone()
                        .into_iter()
                        .enumerate()
                        .map(|(i, model)| {
                            mouse_area(
                                pick_list(
                                    DATA.read().unwrap().models.clone(),
                                    Some(model),
                                    move |x| {
                                        Message::HomePaneView(HomePaneViewMessage::Chats(
                                            id,
                                            ChatsViewMessage::ChangeModel(i, x),
                                        ))
                                    },
                                )
                                .style(style::pick_list::main)
                                .menu_style(style::menu::main)
                                .text_size(BODY_SIZE),
                            )
                            .on_right_press(Message::HomePaneView(HomePaneViewMessage::Chats(
                                id,
                                ChatsViewMessage::RemoveModel(i),
                            )))
                            .into()
                        }))
                    .spacing(5)
                )
                .width(Length::Fill)
                .direction(scrollable::Direction::Horizontal(
                    scrollable::Scrollbar::new()
                )),
                btn_small("add.svg").on_press(Message::HomePaneView(HomePaneViewMessage::Chats(
                    id,
                    ChatsViewMessage::AddModel,
                ))),
            ]
            .spacing(10)
            .align_y(Vertical::Center),
        )
        .width(Length::Fill)
        .align_y(Vertical::Center)
        .style(style::container::bottom_input_back);

        let input = container(
            if self.files.is_empty() {
                column![models, self.view_commands(id.clone()), bottom,]
            } else {
                column![files, models, self.view_commands(id.clone()), bottom,]
            }
            .spacing(10),
        )
        .width(Length::Fill)
        .padding(Padding::from([10, 20]))
        .style(style::container::input_back_opaque);

        let input = container(input).padding(10);

        let body: Element<'a, Message> = match self.messages.is_empty() {
            true => container(self.view_start(id))
                .height(Length::Fill)
                .width(Length::Fill)
                .into(),
            false => {
                let mut col = column(self.messages.iter().map(|x| {
                    Self::view_message(
                        id.clone(),
                        &app.theme(),
                        app.cache.home_shared.messages.0.get(x).unwrap(),
                        self.edits.get(x),
                        self.expanded_messages.contains(&x),
                    )
                    .into()
                }))
                .spacing(20);

                col = col.push(space().height(if self.files.is_empty() { 130 } else { 250 }));
                scrollable::Scrollable::new(col)
                    .direction(scrollable::Direction::Vertical(scrollable::Scrollbar::new()))
                    .anchor_bottom()
                    .height(Length::Fill)
                    .into()
            }
        };

        container(stack([body, column![space::vertical(), input].into()])).into()
    }

    fn view_commands<'a>(&'a self, id: u32) -> Element<'a, Message> {
        container(
            scrollable::Scrollable::new(row(self.prompts.0.iter().map(|x| {
                let chosen = if let Some(y) = &self.selected_prompt {
                    y == &x.id.key().to_string()
                } else {
                    false
                };

                button(text(&x.command).size(SMALL_SIZE))
                    .width(Length::Fill)
                    .style(if chosen {
                        style::button::chosen_chat
                    } else {
                        style::button::not_chosen_chat
                    })
                    .on_press(Message::HomePaneView(HomePaneViewMessage::Chats(
                        id,
                        ChatsViewMessage::ApplyPrompt(Some(x.id.key().to_string())),
                    )))
                    .padding(10)
                    .into()
            })))
            .width(Length::Fill),
        )
        .max_height(250)
        .into()
    }

    fn view_start<'a>(&'a self, id: u32) -> Element<'a, Message> {
        let title = text("How can I help?")
            .font(get_bold_font())
            .size(HEADER_SIZE)
            .style(style::text::primary)
            .align_x(Horizontal::Left);

        let header = row(start::SECTIONS
            .iter()
            .enumerate()
            .map(|(i, x)| {
                let style = match i == self.start {
                    true => style::button::start_chosen,
                    false => style::button::start,
                };

                button(
                    text(x.title)
                        .style(style::text::translucent::primary)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
                        .size(SUB_HEADING_SIZE),
                )
                .padding(10)
                .style(style)
                .on_press(Message::HomePaneView(HomePaneViewMessage::Chats(
                    id,
                    ChatsViewMessage::ChangeStart(i),
                )))
                .into()
            })
            .collect::<Vec<Element<Message>>>())
        .spacing(10);

        let section: Section = start::SECTIONS[self.start].clone();

        let prompts = column(
            section
                .prompts
                .iter()
                .map(|x| {
                    button(
                        text(*x)
                            .style(style::text::translucent::text)
                            .align_x(Horizontal::Left)
                            .width(Length::Fill)
                            .size(SUB_HEADING_SIZE),
                    )
                    .padding(10)
                    .style(style::button::transparent_translucent)
                    .on_press(Message::HomePaneView(HomePaneViewMessage::Chats(
                        id,
                        ChatsViewMessage::InputAction(text_editor::Action::Edit(
                            text_editor::Edit::Paste(Arc::new(x.to_string())),
                        )),
                    )))
                    .into()
                })
                .collect::<Vec<Element<Message>>>(),
        );

        center(
            container(
                column![
                    title,
                    rule::horizontal(1).style(style::rule::translucent::primary),
                    header,
                    rule::horizontal(1).style(style::rule::translucent::text),
                    prompts
                ]
                .spacing(20)
                .align_x(Horizontal::Left),
            )
            .max_width(800)
            .padding(Padding::new(20.0))
            .style(style::container::neutral_back),
        )
        .into()
    }
}
