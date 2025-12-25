use crate::{
    Application, DATA, InputMessage, Message,
    pages::{
        PageMessage,
        home::{HomePaneType, panes::PaneMessage, sidebar::SideBarItems},
    },
    windows::message::WindowMessage,
};
use iced::{Task, window};
use ochat_common::data::RequestType;
use ochat_types::{
    chats::{Chat, ChatData, previews::Preview},
    folders::{Folder, FolderData, FolderDataBuilder},
};

#[derive(Debug, Clone)]
pub enum HomePickingType {
    ReplaceChat(String),
    OpenPane(HomePaneType),
}

#[derive(Debug, Clone)]
pub enum HomeMessage {
    SearchItems(InputMessage),
    SetItems(SideBarItems),
    ExpandItem(String),
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
            Self::SearchItems(InputMessage::Update(x)) => {
                let page = app.get_home_page(&id).unwrap();
                if x.is_empty() {
                    page.side_bar.items.0.clear();
                }
                page.side_bar.search = x;
                Task::none()
            }
            Self::SplitDrag(x) => {
                app.get_home_page(&id).unwrap().side_bar.split =
                    app.windows.get(&id).unwrap().get_size_from_split(x);
                Task::none()
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
                    Ok(_) => Message::Cache(crate::CacheMessage::ResetSideBarItems),
                    Err(_) => match req
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
        }
    }
}
