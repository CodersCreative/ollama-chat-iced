use crate::{
    Application, DATA, InputMessage, Message,
    font::{HEADER_SIZE, SUB_HEADING_SIZE, get_bold_font},
    pages::{
        PageMessage,
        home::{
            COLLAPSED_CUT_OFF, HomePaneType, NORMAL_SIZE,
            message::{HomeMessage, HomePickingType},
            panes::PaneMessage,
        },
    },
    style::{self},
    utils::get_path_assets,
    windows::message::WindowMessage,
};
use iced::{
    Element, Length, Padding, Theme,
    alignment::{Horizontal, Vertical},
    widget::{
        Button, button, column, container, hover, markdown, right, row, rule, space, svg, text,
        text_input,
    },
    window,
};
use ochat_common::data::RequestType;
use ochat_types::{chats::previews::Preview, folders::Folder, surreal::RecordId};

#[derive(Debug, Clone)]
pub struct PreviewMk {
    pub markdown: Vec<markdown::Item>,
    pub id: RecordId,
}

#[derive(Debug, Clone)]
pub struct FolderMk {
    pub name: String,
    pub markdown: Vec<markdown::Item>,
    pub chats: Vec<String>,
    pub id: RecordId,
}

impl From<Preview> for PreviewMk {
    fn from(value: Preview) -> Self {
        Self {
            markdown: markdown::parse(&if value.text.is_empty() {
                "Generation Failed".to_string()
            } else {
                value.text.clone()
            })
            .collect(),
            id: value.id,
        }
    }
}

impl From<Folder> for FolderMk {
    fn from(value: Folder) -> Self {
        Self {
            markdown: markdown::parse(&if value.name.is_empty() {
                "Unknown".to_string()
            } else {
                value.name.clone()
            })
            .collect(),
            id: value.id,
            name: value.name,
            chats: value.chats,
        }
    }
}
#[derive(Debug, Clone)]
pub enum SideBarItem {
    Preview(PreviewMk),
    Folder {
        folder: FolderMk,
        children: Vec<Self>,
    },
}
#[derive(Debug, Clone, Default)]
pub struct SideBarItems(pub Vec<SideBarItem>);

impl SideBarItem {
    pub fn get_record_id(&self) -> &RecordId {
        match self {
            Self::Preview(x) => &x.id,
            Self::Folder {
                folder,
                children: _,
            } => &folder.id,
        }
    }

    pub fn get_first_preview(&self) -> Option<&PreviewMk> {
        match self {
            Self::Preview(x) => Some(x),
            Self::Folder {
                folder: _,
                children,
            } => {
                let mut ret = None;
                for child in children {
                    ret = child.get_first_preview();
                    if ret.is_some() {
                        break;
                    }
                }
                ret
            }
        }
    }

    pub fn is_folder(&self) -> bool {
        if let Self::Preview(_) = self {
            false
        } else {
            true
        }
    }

    pub fn get_markdown(&self) -> &[markdown::Item] {
        match self {
            Self::Preview(x) => &x.markdown,
            Self::Folder {
                folder,
                children: _,
            } => &folder.markdown,
        }
    }

    pub fn update_from_id<F: FnMut(&mut Self)>(&mut self, id: &RecordId, func: &mut F) {
        match self {
            SideBarItem::Preview(x) if &x.id == id => func(self),
            SideBarItem::Folder {
                folder,
                children: _,
            } if &folder.id == id => func(self),
            SideBarItem::Folder {
                folder: _,
                children,
            } => {
                for child in children.iter_mut() {
                    child.update_from_id(id, func);
                }
            }
            _ => {}
        }
    }
    pub fn get_children_recursive(
        &self,
        all_folders: &[Folder],
        all_previews: &[Preview],
    ) -> Vec<Self> {
        let (current_id, chat_ids) = match self {
            Self::Folder { folder, .. } => (folder.id.key().to_string(), &folder.chats),
            _ => return Vec::new(),
        };

        let mut children: Vec<Self> = all_folders
            .iter()
            .filter(|f| f.parent.as_ref() == Some(&current_id))
            .map(|f| {
                let mut item = Self::Folder {
                    folder: f.clone().into(),
                    children: Vec::new(),
                };
                let grand_children = item.get_children_recursive(all_folders, all_previews);
                if let Self::Folder { children, .. } = &mut item {
                    *children = grand_children;
                }
                item
            })
            .collect();

        let mut preview_children: Vec<Self> = all_previews
            .iter()
            .filter(|p| chat_ids.contains(&p.id.key().to_string()))
            .map(|p| Self::Preview(p.clone().into()))
            .collect();

        children.append(&mut preview_children);
        children
    }

    pub fn contains(&self, id: &RecordId) -> bool {
        match self {
            Self::Preview(x) => &x.id == id,
            Self::Folder {
                folder,
                children: _,
            } if &folder.id == id => true,
            Self::Folder {
                folder: _,
                children,
            } => {
                let mut contains = false;
                for child in children {
                    if child.contains(id) {
                        contains = true;
                        break;
                    }
                }
                contains
            }
        }
    }
    pub fn contains_any(&self, wanted: &[RecordId]) -> bool {
        match self {
            Self::Preview(x) => wanted.contains(&x.id),
            Self::Folder {
                folder,
                children: _,
            } if wanted.contains(&folder.id) => true,
            Self::Folder {
                folder: _,
                children,
            } => {
                let mut contains = false;
                for child in children {
                    if child.contains_any(wanted) {
                        contains = true;
                        break;
                    }
                }
                contains
            }
        }
    }

    pub fn filter_contains_any(&mut self, wanted: &[RecordId]) {
        if let Self::Folder {
            folder: _,
            children,
        } = self
        {
            children.retain(|x| x.contains_any(wanted));

            for child in children.iter_mut() {
                child.filter_contains_any(wanted);
            }
        }
    }
}

impl SideBarItems {
    pub fn update_from_id<F: FnMut(&mut SideBarItem)>(&mut self, id: &RecordId, func: &mut F) {
        for child in self.0.iter_mut() {
            child.update_from_id(id, func);
        }
    }

    pub fn get_first_preview(&self) -> Option<&PreviewMk> {
        let mut ret = None;
        for child in self.0.iter() {
            ret = child.get_first_preview();
            if ret.is_some() {
                break;
            }
        }
        ret
    }

    pub async fn get(search: Option<String>) -> Result<Self, String> {
        let req = DATA.read().unwrap().to_request();

        let mut list = {
            let mut list = Vec::new();
            let previews = req
                .make_request::<Vec<Preview>, ()>("preview/all/", &(), RequestType::Get)
                .await?;
            let folders = req
                .make_request::<Vec<Folder>, ()>("folder/all/", &(), RequestType::Get)
                .await?;

            for folder_data in folders.iter().filter(|x| x.parent.is_none()) {
                let mut folder_item = SideBarItem::Folder {
                    folder: folder_data.clone().into(),
                    children: Vec::new(),
                };

                let children = folder_item.get_children_recursive(&folders, &previews);

                if let SideBarItem::Folder {
                    children: folder_children,
                    ..
                } = &mut folder_item
                {
                    *folder_children = children;
                }

                list.push(folder_item);
            }

            let mut previews = previews
                .into_iter()
                .filter(|x| {
                    folders
                        .iter()
                        .position(|y| y.chats.contains(&x.id.key().to_string()))
                        .is_none()
                })
                .map(|x| SideBarItem::Preview(x.into()))
                .collect();

            list.append(&mut previews);

            list
        };

        if let Some(search) = search {
            let mut wanted: Vec<RecordId> = req
                .make_request::<Vec<Preview>, ()>(
                    &format!("preview/search/{}", search),
                    &(),
                    RequestType::Get,
                )
                .await?
                .into_iter()
                .map(|x| x.id)
                .collect();

            wanted.append(
                &mut req
                    .make_request::<Vec<Folder>, ()>(
                        &format!("folder/search/{}", search),
                        &(),
                        RequestType::Get,
                    )
                    .await?
                    .into_iter()
                    .map(|x| x.id)
                    .collect(),
            );

            list.retain(|x| x.contains_any(&wanted));

            for child in list.iter_mut() {
                child.filter_contains_any(&wanted);
            }
        }
        Ok(Self(list))
    }
}

#[derive(Debug, Clone)]
pub struct HomeSideBar {
    pub split: f32,
    pub is_collapsed: bool,
    pub items: SideBarItems,
    pub expanded: Vec<String>,
    pub search: String,
}

impl Default for HomeSideBar {
    fn default() -> Self {
        Self {
            split: NORMAL_SIZE,
            is_collapsed: false,
            expanded: Vec::new(),
            items: SideBarItems::default(),
            search: String::new(),
        }
    }
}

impl HomeSideBar {
    pub fn view<'a>(&'a self, app: &'a Application, id: window::Id) -> Element<'a, Message> {
        let mut content = row![self.pane_buttons(app, id.clone())];
        if self.split > COLLAPSED_CUT_OFF || !self.is_collapsed {
            content = content.push(self.chat_buttons(app, id));
        }

        content = content.push(rule::vertical(2).style(style::rule::side_bar_darker));

        container(content).style(style::container::side_bar).into()
    }

    fn view_item<'a>(
        &self,
        id: window::Id,
        item: &'a SideBarItem,
        parent: Option<String>,
        expanded: bool,
        theme: &Theme,
    ) -> Element<'a, Message> {
        let title = button(markdown::view_with(
            item.get_markdown().iter(),
            style::markdown::main(theme),
            &style::markdown::CustomViewer,
        ))
        .clip(true)
        .on_press({
            if item.is_folder() {
                Message::Window(WindowMessage::Page(
                    id,
                    PageMessage::Home(HomeMessage::ExpandItem(
                        item.get_record_id().key().to_string(),
                    )),
                ))
            } else {
                Message::Window(WindowMessage::Page(
                    id,
                    PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                        HomePickingType::ReplaceChat(item.get_record_id().key().to_string()),
                    ))),
                ))
            }
        })
        .style(style::button::transparent_back_white_text)
        .width(Length::Fill);

        let mut hover_buttons = row![
            style::svg_button::text("close.svg", SUB_HEADING_SIZE)
                .height(Length::Fill)
                .on_press(Message::Window(WindowMessage::Page(
                    id,
                    PageMessage::Home(HomeMessage::DeleteItem(
                        item.get_record_id().key().to_string(),
                    )),
                )))
        ]
        .align_y(Vertical::Center)
        .spacing(5);

        if item.is_folder() {
            if parent.is_some() {
                hover_buttons = hover_buttons.push(
                    style::svg_button::text("folder_upload.svg", SUB_HEADING_SIZE)
                        .height(Length::Fill)
                        .on_press(Message::Window(WindowMessage::Page(
                            id,
                            PageMessage::Home(HomeMessage::RemoveFolderFromFolder(
                                item.get_record_id().key().to_string(),
                            )),
                        ))),
                );
            }
            hover_buttons = hover_buttons.push(
                style::svg_button::text("folder_new.svg", SUB_HEADING_SIZE)
                    .height(Length::Fill)
                    .on_press(Message::Window(WindowMessage::Page(
                        id,
                        PageMessage::Home(HomeMessage::NewFolderToFolder(
                            item.get_record_id().key().to_string(),
                        )),
                    ))),
            );

            hover_buttons = hover_buttons.push(
                style::svg_button::text("add.svg", SUB_HEADING_SIZE)
                    .height(Length::Fill)
                    .on_press(Message::Window(WindowMessage::Page(
                        id,
                        PageMessage::Home(HomeMessage::NewChatToFolder(
                            item.get_record_id().key().to_string(),
                        )),
                    ))),
            );
        } else {
            if let Some(parent) = parent {
                hover_buttons = hover_buttons.push(
                    style::svg_button::text("folder_upload.svg", SUB_HEADING_SIZE)
                        .height(Length::Fill)
                        .on_press(Message::Window(WindowMessage::Page(
                            id,
                            PageMessage::Home(HomeMessage::RemoveChatFromFolder(
                                parent,
                                item.get_record_id().key().to_string(),
                            )),
                        ))),
                );
            }

            hover_buttons = hover_buttons.push(
                style::svg_button::text("zip.svg", SUB_HEADING_SIZE)
                    .height(Length::Fill)
                    .on_press(Message::Window(WindowMessage::Page(
                        id,
                        PageMessage::Home(HomeMessage::ArchiveChat(
                            item.get_record_id().key().to_string(),
                        )),
                    ))),
            );

            hover_buttons = hover_buttons.push(
                style::svg_button::text("thumbs_up.svg", SUB_HEADING_SIZE)
                    .height(Length::Fill)
                    .on_press(Message::Window(WindowMessage::Page(
                        id,
                        PageMessage::Home(HomeMessage::FavChat(
                            item.get_record_id().key().to_string(),
                        )),
                    ))),
            );
        }

        let mut body = column![
            container(hover(
                row(if let SideBarItem::Folder { folder, .. } = item {
                    vec![
                        svg(svg::Handle::from_path(get_path_assets(
                            match folder.name.as_str() {
                                "Archived" => "zip.svg".to_string(),
                                "Favourites" => "thumbs_up.svg".to_string(),
                                _ => "folder.svg".to_string(),
                            },
                        )))
                        .style(style::svg::text)
                        .width(SUB_HEADING_SIZE)
                        .into(),
                        title.into(),
                    ]
                } else {
                    vec![title.into()]
                })
                .align_y(Vertical::Center)
                .spacing(5),
                right(hover_buttons).align_y(Vertical::Center)
            ))
            .max_height(HEADER_SIZE * 2)
        ]
        .spacing(10);

        if expanded {
            if let SideBarItem::Folder {
                folder: _,
                children,
            } = item
            {
                if !children.is_empty() {
                    body = body.push(
                        row![
                            space(),
                            column(children.iter().map(|x| {
                                self.view_item(
                                    id.clone(),
                                    x,
                                    Some(item.get_record_id().key().to_string()),
                                    self.expanded.contains(&x.get_record_id().key().to_string()),
                                    theme,
                                )
                            }))
                            .spacing(5)
                        ]
                        .spacing(5),
                    );
                    body = body.push(rule::horizontal(1).style(style::rule::translucent::text))
                }
            }
        }

        body.into()
    }

    fn chat_buttons<'a>(&'a self, app: &'a Application, id: window::Id) -> Element<'a, Message> {
        let name = button(
            text("ochat")
                .font(get_bold_font())
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .width(Length::Fill)
                .style(style::text::primary)
                .size(SUB_HEADING_SIZE),
        )
        .style(style::button::transparent_back_white_text)
        .padding(0)
        .width(Length::Fill)
        .on_press(Message::Window(WindowMessage::Page(
            id,
            PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                HomePickingType::OpenPane(HomePaneType::Info),
            ))),
        )));

        let new_chat = button(
            text("New Chat")
                .font(get_bold_font())
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .width(Length::Fill)
                .size(HEADER_SIZE),
        )
        .on_press(Message::Window(WindowMessage::Page(
            id,
            PageMessage::Home(HomeMessage::NewChat),
        )))
        .style(style::button::rounded_primary_blend)
        .width(Length::Fill)
        .padding(Padding::from(10));

        let new_folder = style::svg_button::text("folder_new.svg", HEADER_SIZE)
            .on_press(Message::Window(WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::NewFolder),
            )))
            .style(style::button::rounded_primary_blend)
            .padding(Padding::from(10));

        let search = style::svg_input::primary(
            Some(String::from("search.svg")),
            text_input("Search chats...", &self.search)
                .on_input(move |x| {
                    Message::Window(WindowMessage::Page(
                        id,
                        PageMessage::Home(HomeMessage::SearchItems(InputMessage::Update(x))),
                    ))
                })
                .on_submit(Message::Window(WindowMessage::Page(
                    id,
                    PageMessage::Home(HomeMessage::SearchItems(InputMessage::Submit)),
                ))),
            SUB_HEADING_SIZE,
        );

        let previews = column(
            if self.search.is_empty() || self.items.0.is_empty() {
                &app.cache.side_bar_items.0
            } else {
                &self.items.0
            }
            .iter()
            .map(|x| {
                self.view_item(
                    id.clone(),
                    x,
                    None,
                    self.expanded.contains(&x.get_record_id().key().to_string()),
                    &app.theme(),
                )
            }),
        )
        .spacing(5);

        container(
            column![
                name,
                row![new_chat, new_folder]
                    .spacing(10)
                    .align_y(Vertical::Center),
                search,
                previews,
                space::vertical()
            ]
            .spacing(10)
            .padding(10),
        )
        .into()
    }

    fn pane_buttons_vec<'a>(&'a self, id: window::Id, size: u32) -> Vec<Button<'a, Message>> {
        let new_chat_pane = style::svg_button::text("add_chat.svg", size).on_press(
            Message::Window(WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                    HomePickingType::OpenPane(HomePaneType::Chat),
                ))),
            )),
        );

        let new_models_pane = style::svg_button::text("star.svg", size).on_press(Message::Window(
            WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                    HomePickingType::OpenPane(HomePaneType::Models),
                ))),
            ),
        ));

        let new_prompts_pane = style::svg_button::text("prompt.svg", size).on_press(
            Message::Window(WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                    HomePickingType::OpenPane(HomePaneType::Prompts),
                ))),
            )),
        );

        let new_tools_pane = style::svg_button::text("tools.svg", size).on_press(Message::Window(
            WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                    HomePickingType::OpenPane(HomePaneType::Tools),
                ))),
            ),
        ));

        let new_options_pane =
            style::svg_button::text("ai.svg", size).on_press(Message::Window(WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                    HomePickingType::OpenPane(HomePaneType::Options),
                ))),
            )));

        let new_pulls_pane = style::svg_button::text("downloads.svg", size).on_press(
            Message::Window(WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                    HomePickingType::OpenPane(HomePaneType::Pulls),
                ))),
            )),
        );

        let new_settings_pane = style::svg_button::text("settings.svg", size).on_press(
            Message::Window(WindowMessage::Page(
                id,
                PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                    HomePickingType::OpenPane(HomePaneType::Settings),
                ))),
            )),
        );

        let quit = style::svg_button::danger("quit.svg", size)
            .on_press(Message::Window(WindowMessage::CloseWindow(id)));

        vec![
            new_chat_pane,
            new_models_pane,
            new_prompts_pane,
            new_tools_pane,
            new_options_pane,
            new_pulls_pane,
            new_settings_pane,
            quit,
        ]
    }

    fn pane_buttons<'a>(&'a self, _app: &'a Application, id: window::Id) -> Element<'a, Message> {
        let size = 24;

        let collapse = style::svg_button::text(
            if self.is_collapsed {
                "panel_open.svg"
            } else {
                "panel_close.svg"
            },
            size,
        )
        .on_press(Message::Window(WindowMessage::Page(
            id,
            PageMessage::Home(HomeMessage::CollapseSideBar),
        )));

        let new_chat = style::svg_button::text("add.svg", size).on_press(Message::Window(
            WindowMessage::Page(id, PageMessage::Home(HomeMessage::NewChat)),
        ));

        let mut col = column![collapse, new_chat, space::vertical()]
            .spacing(5)
            .padding(Padding::default().top(5).bottom(5));

        for button in self.pane_buttons_vec(id, size) {
            col = col.push(button);
        }

        container(col)
            .style(style::container::side_bar_darker)
            .into()
    }
}
