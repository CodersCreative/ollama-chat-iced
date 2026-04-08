use crate::{
    Application, DATA, InputMessage, Message,
    font::{HEADER_SIZE, SUB_HEADING_SIZE, get_bold_font},
    pages::{
        PageMessage,
        home::{
            BUTTON_COLLAPSE_CUT_OFF, BUTTONS_EXPAND_CUT_OFF, COLLAPSED_CUT_OFF, HomePaneType,
            NORMAL_SIZE,
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
        Button, Id as WidgetId, button, center_x, column, container, hover, markdown, mouse_area,
        right, row, rule, space, svg, text_input,
    },
    window::{self},
};
use iced_drop::droppable;
use iced_selection::text;
use ochat_common::data::RequestType;
use ochat_types::{chats::previews::Preview, folders::Folder, surreal::RecordId};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PreviewMk {
    pub name: String,
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
            name: value.text.clone(),
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
pub struct SideBarItems {
    pub items: Vec<SideBarItem>,
}

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

    pub fn get_name(&self) -> &str {
        match self {
            Self::Preview(x) => &x.name,
            Self::Folder {
                folder,
                children: _,
            } => &folder.name,
        }
    }

    pub fn is_folder(&self) -> bool {
        if let Self::Preview(_) = self {
            false
        } else {
            true
        }
    }

    pub fn is_builtin(&self) -> bool {
        if let Self::Folder {
            folder,
            children: _,
        } = self
        {
            ["Favourites", "Archived"].contains(&folder.name.trim())
        } else {
            false
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

    pub fn folder_ids(&self) -> Vec<String> {
        match self {
            Self::Folder { folder, children } => {
                let mut lst = vec![folder.id.key().to_string()];
                for child in children {
                    lst.append(&mut child.folder_ids());
                }
                lst
            }
            Self::Preview(_) => Vec::new(),
        }
    }

    pub fn get_children_recursive(
        &self,
        all_folders: &[Folder],
        all_previews: &[Preview],
    ) -> Vec<Self> {
        let (current_id, chat_ids) = match self {
            Self::Folder { folder, .. } => (
                folder.id.key().to_string().trim().to_string(),
                &folder.chats,
            ),
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
            .filter(|p| chat_ids.contains(&p.id.key().to_string().trim().to_string()))
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

    pub fn find_parent_id(&self, target: &str, parent: Option<&str>) -> Option<String> {
        match self {
            SideBarItem::Preview(x) => {
                if x.id.key().to_string() == target {
                    parent.map(|p| p.to_string())
                } else {
                    None
                }
            }
            SideBarItem::Folder { folder, children } => {
                let current_id = folder.id.key().to_string();
                if current_id == target {
                    parent.map(|p| p.to_string())
                } else {
                    for child in children {
                        if let Some(found) = child.find_parent_id(target, Some(&current_id)) {
                            return Some(found);
                        }
                    }
                    None
                }
            }
        }
    }

    pub fn find_folder_name(&self, target: &str) -> Option<String> {
        match self {
            SideBarItem::Folder { folder, children } => {
                if folder.id.key().to_string() == target {
                    Some(folder.name.clone())
                } else {
                    for child in children {
                        if let Some(found) = child.find_folder_name(target) {
                            return Some(found);
                        }
                    }
                    None
                }
            }
            _ => None,
        }
    }
}

impl SideBarItems {
    pub fn folder_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();

        for child in &self.items {
            ids.append(&mut child.folder_ids());
        }

        ids
    }
    pub fn update_from_id<F: FnMut(&mut SideBarItem)>(&mut self, id: &RecordId, func: &mut F) {
        for child in self.items.iter_mut() {
            child.update_from_id(id, func);
        }
    }

    pub fn get_first_preview(&self) -> Option<&PreviewMk> {
        let mut ret = None;
        for child in self.items.iter() {
            ret = child.get_first_preview();
            if ret.is_some() {
                break;
            }
        }
        ret
    }

    pub async fn get(search: Option<String>) -> Result<Self, String> {
        let req = DATA.read().unwrap().to_request();

        let mut items = {
            let mut list = Vec::new();
            let previews = req
                .make_request::<Vec<Preview>, ()>("preview/all/", &(), RequestType::Get)
                .await?;
            let folders = req
                .make_request::<Vec<Folder>, ()>("folder/all/", &(), RequestType::Get)
                .await?;

            for folder_data in folders.iter().filter(|x| x.parent.is_none()).cloned() {
                let mut folder_item = SideBarItem::Folder {
                    folder: folder_data.into(),
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
                        .filter(|x| !["Favourites"].contains(&x.name.as_str()))
                        .position(|y| y.chats.contains(&x.id.key().to_string().trim().to_string()))
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

            items.retain(|x| x.contains_any(&wanted));

            for child in items.iter_mut() {
                child.filter_contains_any(&wanted);
            }
        }
        Ok(Self { items })
    }

    pub fn parent_id_of(&self, target: &str) -> Option<String> {
        for item in &self.items {
            if let Some(found) = item.find_parent_id(target, None) {
                return Some(found);
            }
        }
        None
    }

    pub fn folder_name_by_id(&self, target: &str) -> Option<String> {
        for item in &self.items {
            if let Some(found) = item.find_folder_name(target) {
                return Some(found);
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct HomeSideBar {
    pub split: f32,
    pub is_collapsed: bool,
    pub items: SideBarItems,
    pub expanded: Vec<String>,
    pub buttons_expanded: Vec<String>,
    pub editing: HashMap<String, String>,
    pub dragging: Option<DragItem>,
    pub drag_hover: Option<WidgetId>,
    pub search: String,
}

#[derive(Debug, Clone)]
pub enum DragItem {
    Chat(String),
    Folder(String),
}

impl DragItem {
    pub fn id(&self) -> &str {
        match self {
            Self::Chat(id) | Self::Folder(id) => id,
        }
    }
}

pub const SIDEBAR_ROOT_ID: &str = "sidebar-root";
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum IsInSpecial {
    Fav,
    Archive,
    None,
}

impl Default for HomeSideBar {
    fn default() -> Self {
        Self {
            split: NORMAL_SIZE,
            is_collapsed: false,
            dragging: None,
            drag_hover: None,
            expanded: Vec::new(),
            buttons_expanded: Vec::new(),
            editing: HashMap::new(),
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

        container(content)
            .id(WidgetId::from(SIDEBAR_ROOT_ID))
            .style(style::container::side_bar)
            .into()
    }

    fn view_item<'a>(
        &'a self,
        id: window::Id,
        item: &'a SideBarItem,
        _parent: Option<String>,
        expanded: bool,
        buttons_expanded: bool,
        edit: Option<&'a String>,
        is_in_special: IsInSpecial,
        theme: &Theme,
    ) -> Element<'a, Message> {
        let title: Element<'a, Message> = if let Some(edit) = edit {
            text_input("Input folder title...", edit)
                .on_input(move |x| {
                    Message::Window(WindowMessage::Page(
                        id,
                        PageMessage::Home(HomeMessage::EditFolderMessage(
                            item.get_record_id().key().to_string(),
                            InputMessage::Update(x),
                        )),
                    ))
                })
                .on_submit(Message::Window(WindowMessage::Page(
                    id,
                    PageMessage::Home(HomeMessage::EditFolderMessage(
                        item.get_record_id().key().to_string(),
                        InputMessage::Submit,
                    )),
                )))
                .style(style::text_input::input)
                .width(Length::Fill)
                .into()
        } else {
            container(markdown::view_with(
                item.get_markdown().iter(),
                style::markdown::main(theme),
                &style::markdown::CustomViewer,
            ))
            .padding(5)
            .width(Length::Fill)
            .into()
        };

        let mut hover_buttons = row![].align_y(Vertical::Center).spacing(5);

        if edit.is_some() {
            hover_buttons = hover_buttons.push(
                style::svg_button::text("close.svg", SUB_HEADING_SIZE)
                    .height(Length::Fill)
                    .on_press(Message::Window(WindowMessage::Page(
                        id,
                        PageMessage::Home(HomeMessage::EditFolder(
                            item.get_record_id().key().to_string(),
                        )),
                    ))),
            );
        } else if item.is_folder() {
            if !buttons_expanded && self.split < BUTTONS_EXPAND_CUT_OFF {
                hover_buttons = hover_buttons.push(
                    style::svg_button::text("panel_close.svg", SUB_HEADING_SIZE)
                        .height(Length::Fill)
                        .on_press(Message::Window(WindowMessage::Page(
                            id,
                            PageMessage::Home(HomeMessage::ButtonExpandItem(
                                item.get_record_id().key().to_string(),
                            )),
                        ))),
                );
            } else {
                if self.split < BUTTONS_EXPAND_CUT_OFF {
                    hover_buttons = hover_buttons.push(
                        style::svg_button::text("panel_open.svg", SUB_HEADING_SIZE)
                            .height(Length::Fill)
                            .on_press(Message::Window(WindowMessage::Page(
                                id,
                                PageMessage::Home(HomeMessage::ButtonExpandItem(
                                    item.get_record_id().key().to_string(),
                                )),
                            ))),
                    );
                }

                if !item.is_builtin() {
                    hover_buttons = hover_buttons.push(
                        style::svg_button::text("edit.svg", SUB_HEADING_SIZE)
                            .height(Length::Fill)
                            .on_press(Message::Window(WindowMessage::Page(
                                id,
                                PageMessage::Home(HomeMessage::EditFolder(
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
            }
        } else if is_in_special == IsInSpecial::None {
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

        let item_id = item.get_record_id().key().to_string();

        let header = container(
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
                    title,
                ]
            } else {
                vec![title]
            })
            .align_y(Vertical::Center)
            .spacing(5)
            .width(Length::Fill),
        )
        .max_height(HEADER_SIZE * 2)
        .style(if self.dragging.is_some() && item.is_folder() {
            style::container::drop_target
        } else {
            iced::widget::container::transparent
        })
        .id(format!(
            "{}:{}",
            if item.is_folder() { "folder" } else { "chat" },
            item_id.trim()
        ));

        let press_message = if item.is_folder() {
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
        };

        let header: Element<'a, Message> = if item.is_builtin() || edit.is_some() {
            mouse_area(header).on_press(press_message).into()
        } else {
            let drag_item = if item.is_folder() {
                DragItem::Folder(item_id.clone())
            } else {
                DragItem::Chat(item_id.clone())
            };

            droppable(header)
                .on_press(press_message)
                .on_drag({
                    let drag_item = drag_item.clone();
                    move |point, _| {
                        Message::Window(WindowMessage::Page(
                            id,
                            PageMessage::Home(HomeMessage::DragMove(drag_item.clone(), point)),
                        ))
                    }
                })
                .on_drop(move |point, _| {
                    Message::Window(WindowMessage::Page(
                        id,
                        PageMessage::Home(HomeMessage::Drop(drag_item.clone(), point)),
                    ))
                })
                .on_cancel(Message::Window(WindowMessage::Page(
                    id,
                    PageMessage::Home(HomeMessage::CancelDrag),
                )))
                .drag_threshold(0.0)
                .drag_center(true)
                .drag_overlay(true)
                .into()
        };

        let header: Element<'a, Message> = container(hover(
            header,
            right(hover_buttons).align_y(Vertical::Center),
        ))
        .max_height(HEADER_SIZE * 2)
        .into();

        let mut body = column![header].spacing(10);

        if expanded
            && let SideBarItem::Folder { folder, children } = item
            && !children.is_empty()
        {
            body = body.push(
                container(
                    row![
                        space().width(10),
                        column(children.iter().map(|x| {
                            let child_id = x.get_record_id().key().to_string();
                            self.view_item(
                                id.clone(),
                                x,
                                Some(item.get_record_id().key().to_string()),
                                self.expanded.contains(&child_id),
                                self.buttons_expanded.contains(&child_id),
                                self.editing.get(&child_id),
                                match folder.name.as_str() {
                                    "Archived" => IsInSpecial::Archive,
                                    "Favourites" => IsInSpecial::Fav,
                                    _ => is_in_special.clone(),
                                },
                                theme,
                            )
                        }))
                        .spacing(0)
                    ]
                    .spacing(5),
                )
                .padding(Padding::from([6.0, 0.0]))
                .style(iced::widget::container::transparent),
            );
            body = body.push(rule::horizontal(1).style(style::rule::translucent::text))
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

        let new_chat = if self.split < BUTTON_COLLAPSE_CUT_OFF {
            style::svg_button::text("add.svg", HEADER_SIZE)
        } else {
            button(
                text("New Chat")
                    .font(get_bold_font())
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .width(Length::Fill)
                    .size(HEADER_SIZE),
            )
            .width(Length::Fill)
        }
        .on_press(Message::Window(WindowMessage::Page(
            id,
            PageMessage::Home(HomeMessage::NewChat),
        )))
        .style(style::button::rounded_primary_blend)
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
            if self.search.is_empty() || self.items.items.is_empty() {
                &app.cache.side_bar_items.items
            } else {
                &self.items.items
            }
            .iter()
            .map(|x| {
                let item_id = x.get_record_id().key().to_string();
                self.view_item(
                    id.clone(),
                    x,
                    None,
                    self.expanded.contains(&item_id),
                    self.buttons_expanded.contains(&item_id),
                    self.editing.get(&item_id),
                    IsInSpecial::None,
                    &app.theme(),
                )
            }),
        )
        .spacing(5);

        container(
            column![
                name,
                center_x(
                    row![new_chat, new_folder]
                        .spacing(10)
                        .align_y(Vertical::Center)
                ),
                search,
                previews,
                space::vertical()
            ]
            .spacing(10)
            .padding(10),
        )
        .into()
    }

    fn pane_buttons_vec<'a>(
        &'a self,
        app: &'a Application,
        id: window::Id,
        size: u32,
    ) -> Vec<Button<'a, Message>> {
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

        let mut widgets = vec![new_chat_pane];

        #[cfg(feature = "sound")]
        {
            if app
                .cache
                .server_features
                .contains(&ochat_types::ServerFeatures::Sound)
            {
                widgets.push(
                    style::svg_button::text("call.svg", size).on_press(Message::Window(
                        WindowMessage::Page(
                            id,
                            PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                                HomePickingType::OpenPane(HomePaneType::Call),
                            ))),
                        ),
                    )),
                );
            }
        }

        let quit = style::svg_button::danger("quit.svg", size)
            .on_press(Message::Window(WindowMessage::CloseWindow(id)));

        widgets.append(&mut vec![
            new_models_pane,
            new_prompts_pane,
            new_tools_pane,
            new_options_pane,
            new_pulls_pane,
            new_settings_pane,
            quit,
        ]);

        widgets
    }

    fn pane_buttons<'a>(&'a self, app: &'a Application, id: window::Id) -> Element<'a, Message> {
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

        for button in self.pane_buttons_vec(app, id, size) {
            col = col.push(button);
        }

        container(col)
            .style(style::container::side_bar_darker)
            .into()
    }
}
