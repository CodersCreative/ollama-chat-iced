use std::collections::HashMap;

use crate::{
    Application, DATA, InputMessage, Message,
    pages::{
        PageMessage,
        home::{
            HomePaneType,
            panes::PaneMessage,
            sidebar::{DragItem, SIDEBAR_ROOT_ID, SideBarItems},
        },
    },
    windows::message::WindowMessage,
};
use iced::{Point, Rectangle, Task, widget::Id as WidgetId, window};
use iced_drop::zones_on_point;
use ochat_common::data::RequestType;
use ochat_types::{
    chats::{Chat, ChatData, previews::Preview},
    folders::{Folder, FolderData, FolderDataBuilder, FolderNameData},
};

#[derive(Debug, Clone)]
pub enum HomePickingType {
    ReplaceChat(String),
    OpenPane(HomePaneType),
}

#[derive(Debug, Clone)]
pub enum HomeMessage {
    Dropped(String, String),
    DragMove(DragItem, Point),
    Drop(DragItem, Point),
    DropZones(DragItem, Vec<(WidgetId, Rectangle)>),
    CancelDrag,
    SearchItems(InputMessage),
    SetItems(SideBarItems),
    ExpandItem(String),
    ButtonExpandItem(String),
    EditFolder(String),
    EditFolderMessage(String, InputMessage),
    NewChat,
    NewChatToFolder(String),
    NewFolderToFolder(String),
    FavChat(String),
    ArchiveChat(String),
    RemoveChatFromFolder(String, String),
    RemoveFolderFromFolder(String),
    NewFolder,
    SplitDrag(f32),
    DeleteItem(String),
    Pane(PaneMessage),
    CollapseSideBar,
}

impl HomeMessage {
    pub fn handle(self, app: &mut Application, id: window::Id) -> Task<Message> {
        match self {
            Self::ExpandItem(x) => {
                let page = app.get_home_page(&id).unwrap();
                if page.side_bar.expanded.contains(&x) {
                    page.side_bar.expanded.retain(|y| y != &x);
                } else {
                    page.side_bar.expanded.push(x);
                }
                Task::none()
            }
            Self::ButtonExpandItem(x) => {
                let page = app.get_home_page(&id).unwrap();
                if page.side_bar.buttons_expanded.contains(&x) {
                    page.side_bar.buttons_expanded.retain(|y| y != &x);
                } else {
                    page.side_bar.buttons_expanded.push(x);
                }
                Task::none()
            }
            Self::EditFolder(x) => {
                let page = app.get_home_page(&id).unwrap();
                if page.side_bar.editing.contains_key(&x) {
                    page.side_bar.editing.retain(|y, _| y != &x);
                } else {
                    page.side_bar.editing.insert(x, String::new());
                }
                Task::none()
            }
            Self::EditFolderMessage(folder_id, InputMessage::Update(x)) => {
                app.get_home_page(&id)
                    .unwrap()
                    .side_bar
                    .editing
                    .iter_mut()
                    .filter(|x| x.0 == &folder_id)
                    .for_each(|y| *y.1 = x.clone());
                Task::none()
            }
            Self::EditFolderMessage(folder_id, _) => {
                let value = app
                    .get_home_page(&id)
                    .unwrap()
                    .side_bar
                    .editing
                    .remove(&folder_id)
                    .unwrap_or(String::from("New Folder"));
                Task::future(async move {
                    let req = DATA.read().unwrap().to_request();
                    match req
                        .make_request::<Folder, FolderNameData>(
                            &format!("folder/{}/name/", folder_id),
                            &FolderNameData { name: value },
                            RequestType::Put,
                        )
                        .await
                    {
                        Ok(_) => Message::Cache(crate::CacheMessage::ResetSideBarItems),
                        Err(e) => Message::Err(e),
                    }
                })
            }
            Self::SearchItems(InputMessage::Update(x)) => {
                let page = app.get_home_page(&id).unwrap();
                if x.is_empty() {
                    page.side_bar.items.items.clear();
                }
                page.side_bar.search = x;
                Task::none()
            }
            Self::SplitDrag(x) => {
                app.get_home_page(&id).unwrap().side_bar.split =
                    app.windows.get(&id).unwrap().get_size_from_split(x);
                Task::none()
            }
            Self::DragMove(drag_item, _) => {
                app.get_home_page(&id).unwrap().side_bar.dragging = Some(drag_item.clone());
                Task::none()
            }
            Self::CancelDrag => {
                let page = app.get_home_page(&id).unwrap();
                page.side_bar.dragging = None;
                Task::none()
            }
            Self::Drop(drag_item, point) => zones_on_point(
                move |zones| {
                    Message::Window(WindowMessage::Page(
                        id,
                        PageMessage::Home(HomeMessage::DropZones(drag_item.clone(), zones)),
                    ))
                },
                point,
                None,
                None,
            ),
            Self::DropZones(drag_item, zones) => {
                let using_cache = {
                    let page = app.get_home_page(&id).unwrap();
                    page.side_bar.dragging = None;

                    app.get_home_page(&id)
                        .unwrap()
                        .side_bar
                        .items
                        .items
                        .is_empty()
                };

                let side_bar_items = if using_cache {
                    &app.cache.side_bar_items
                } else {
                    &app.get_home_page(&id).unwrap().side_bar.items
                };

                let mut zone_map = HashMap::new();

                for folder_id in &side_bar_items.folder_ids() {
                    zone_map.insert(
                        WidgetId::from(format!("folder:{}", folder_id)),
                        folder_id.clone(),
                    );
                }

                let in_sidebar = zones
                    .iter()
                    .any(|(id, _)| id == &WidgetId::from(SIDEBAR_ROOT_ID));
                let target = zones
                    .into_iter()
                    .find_map(|(zone_id, _)| zone_map.get(&zone_id).cloned());

                match target {
                    Some(to) if to != drag_item.id() => {
                        let target_name = side_bar_items.folder_name_by_id(&to);
                        match (drag_item.clone(), target_name.as_deref()) {
                            (DragItem::Chat(chat_id), Some("Favourites")) => {
                                Self::FavChat(chat_id).handle(app, id)
                            }
                            (DragItem::Chat(chat_id), Some("Archived")) => {
                                Self::ArchiveChat(chat_id).handle(app, id)
                            }
                            (_, None) => return Task::none(),
                            _ => Self::Dropped(drag_item.id().to_string(), to).handle(app, id),
                        }
                    }
                    _ if !in_sidebar => {
                        Self::DeleteItem(drag_item.id().to_string()).handle(app, id)
                    }
                    _ if in_sidebar => {
                        let parent = side_bar_items.parent_id_of(drag_item.id());
                        match (drag_item, parent) {
                            (DragItem::Chat(chat_id), Some(folder_id)) => {
                                Self::RemoveChatFromFolder(folder_id, chat_id).handle(app, id)
                            }
                            (DragItem::Folder(folder_id), Some(_)) => {
                                Self::RemoveFolderFromFolder(folder_id).handle(app, id)
                            }
                            _ => Task::none(),
                        }
                    }
                    _ => Task::none(),
                }
            }
            Self::SearchItems(_) => {
                let search = app.get_home_page(&id).unwrap().side_bar.search.clone();
                Task::future(async move {
                    match SideBarItems::get(Some(search)).await {
                        Ok(items) => Message::Window(WindowMessage::Page(
                            id,
                            PageMessage::Home(HomeMessage::SetItems(items)),
                        )),
                        Err(e) => Message::Err(e),
                    }
                })
            }
            Self::SetItems(items) => {
                app.get_home_page(&id).unwrap().side_bar.items = items;
                Task::none()
            }
            Self::CollapseSideBar => {
                let page = app.get_home_page(&id).unwrap();
                page.side_bar.is_collapsed = !page.side_bar.is_collapsed;
                Task::none()
            }
            Self::DeleteItem(x) => Task::future(async move {
                let req = DATA.read().unwrap().to_request();
                match req
                    .make_request::<Option<Preview>, ()>(
                        &format!("chat/{}", x),
                        &(),
                        RequestType::Delete,
                    )
                    .await
                {
                    Ok(Some(_)) => Message::Cache(crate::CacheMessage::ResetSideBarItems),
                    _ => match req
                        .make_request::<Option<Folder>, ()>(
                            &format!("folder/{}", x),
                            &(),
                            RequestType::Delete,
                        )
                        .await
                    {
                        Ok(_) => Message::Cache(crate::CacheMessage::ResetSideBarItems),
                        Err(e) => Message::Err(e),
                    },
                }
            }),
            Self::Pane(x) => x.handle(app, id),
            Self::NewChat => Task::future(async move {
                let req = DATA.read().unwrap().to_request();
                match req
                    .make_request::<Chat, ChatData>(
                        "chat/",
                        &ChatData::default(),
                        RequestType::Post,
                    )
                    .await
                {
                    Ok(chat) => Message::Batch(vec![
                        match req
                            .make_request::<Preview, ()>(
                                &format!("preview/{}", chat.id.key().to_string()),
                                &(),
                                RequestType::Put,
                            )
                            .await
                        {
                            Ok(_) => Message::Cache(crate::CacheMessage::ResetSideBarItems),
                            Err(e) => Message::Err(e),
                        },
                        Message::Window(WindowMessage::Page(
                            id,
                            PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                                HomePickingType::ReplaceChat(chat.id.key().to_string()),
                            ))),
                        )),
                    ]),
                    Err(e) => Message::Err(e),
                }
            }),
            Self::FavChat(x) => Task::future(async move {
                let req = DATA.read().unwrap().to_request();
                match req
                    .make_request::<Folder, ()>(
                        &format!("folder/fav/chat/{}", x),
                        &(),
                        RequestType::Put,
                    )
                    .await
                {
                    Ok(_) => Message::Cache(crate::CacheMessage::ResetSideBarItems),
                    Err(e) => Message::Err(e),
                }
            }),
            Self::ArchiveChat(x) => Task::future(async move {
                let req = DATA.read().unwrap().to_request();
                match req
                    .make_request::<Folder, ()>(
                        &format!("folder/archive/chat/{}", x),
                        &(),
                        RequestType::Put,
                    )
                    .await
                {
                    Ok(_) => Message::Cache(crate::CacheMessage::ResetSideBarItems),
                    Err(e) => Message::Err(e),
                }
            }),
            Self::RemoveChatFromFolder(folder, chat) => Task::future(async move {
                let req = DATA.read().unwrap().to_request();
                match req
                    .make_request::<Folder, ()>(
                        &format!("folder/{}/chat/{}", folder, chat),
                        &(),
                        RequestType::Delete,
                    )
                    .await
                {
                    Ok(_) => Message::Cache(crate::CacheMessage::ResetSideBarItems),
                    Err(e) => Message::Err(e),
                }
            }),
            Self::RemoveFolderFromFolder(folder) => Task::future(async move {
                let req = DATA.read().unwrap().to_request();
                match req
                    .make_request::<Folder, ()>(
                        &format!("folder/{}/parent/none", folder),
                        &(),
                        RequestType::Put,
                    )
                    .await
                {
                    Ok(_) => Message::Cache(crate::CacheMessage::ResetSideBarItems),
                    Err(e) => Message::Err(e),
                }
            }),
            Self::NewChatToFolder(x) => Task::future(async move {
                let req = DATA.read().unwrap().to_request();
                match req
                    .make_request::<Chat, ChatData>(
                        "chat/",
                        &ChatData::default(),
                        RequestType::Post,
                    )
                    .await
                {
                    Ok(chat) => Message::Batch(vec![
                        match req
                            .make_request::<Option<Folder>, ()>(
                                &format!("folder/{}/chat/{}", x, chat.id.key().to_string()),
                                &(),
                                RequestType::Put,
                            )
                            .await
                        {
                            Ok(_) => Message::Cache(crate::CacheMessage::ResetSideBarItems),
                            Err(e) => Message::Err(e),
                        },
                        Message::Window(WindowMessage::Page(
                            id,
                            PageMessage::Home(HomeMessage::Pane(PaneMessage::Pick(
                                HomePickingType::ReplaceChat(chat.id.key().to_string()),
                            ))),
                        )),
                    ]),
                    Err(e) => Message::Err(e),
                }
            }),
            Self::NewFolderToFolder(x) => Task::future(async move {
                let req = DATA.read().unwrap().to_request();
                match req
                    .make_request::<Folder, FolderData>(
                        "folder/",
                        &FolderDataBuilder::default().build().unwrap_or_default(),
                        RequestType::Post,
                    )
                    .await
                {
                    Ok(folder) => match req
                        .make_request::<Option<Folder>, ()>(
                            &format!("folder/{}/parent/{}", folder.id.key().to_string(), x),
                            &(),
                            RequestType::Put,
                        )
                        .await
                    {
                        Ok(_) => Message::Cache(crate::CacheMessage::ResetSideBarItems),
                        Err(e) => Message::Err(e),
                    },
                    Err(e) => Message::Err(e),
                }
            }),
            Self::NewFolder => Task::future(async move {
                let req = DATA.read().unwrap().to_request();
                match req
                    .make_request::<Folder, FolderData>(
                        "folder/",
                        &FolderDataBuilder::default().build().unwrap_or_default(),
                        RequestType::Post,
                    )
                    .await
                {
                    Ok(_) => Message::Cache(crate::CacheMessage::ResetSideBarItems),
                    Err(e) => Message::Err(e),
                }
            }),
            Self::Dropped(from, to) => {
                let is_folder = app.cache.side_bar_items.folder_ids().contains(&from);

                Task::future(async move {
                    let req = DATA.read().unwrap().to_request();
                    if is_folder {
                        match req
                            .make_request::<Option<Folder>, ()>(
                                &format!("folder/{}/parent/{}", to, from),
                                &(),
                                RequestType::Put,
                            )
                            .await
                        {
                            Ok(_) => Message::Cache(crate::CacheMessage::ResetSideBarItems),
                            Err(e) => Message::Err(e),
                        }
                    } else {
                        match req
                            .make_request::<Option<Folder>, ()>(
                                &format!("folder/{}/chat/{}", to, from),
                                &(),
                                RequestType::Put,
                            )
                            .await
                        {
                            Ok(_) => Message::Cache(crate::CacheMessage::ResetSideBarItems),
                            Err(e) => Message::Err(e),
                        }
                    }
                })
            }
        }
    }
}
