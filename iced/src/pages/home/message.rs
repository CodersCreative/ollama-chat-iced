use crate::{
    Application, DATA, InputMessage, Message,
    pages::{
        PageMessage, Pages,
        home::{HomePaneType, panes::PaneMessage, sidebar::PreviewMk},
    },
    windows::message::WindowMessage,
};
use iced::{Task, window};
use ochat_common::data::RequestType;
use ochat_types::chats::{Chat, ChatData, previews::Preview};

#[derive(Debug, Clone)]
pub enum HomePickingType {
    ReplaceChat(String),
    OpenPane(HomePaneType),
}

#[derive(Debug, Clone)]
pub enum HomeMessage {
    SearchPreviews(InputMessage),
    SetPreviews(Vec<Preview>),
    NewChat,
    SplitDrag(f32),
    DeleteChat(String),
    Pane(PaneMessage),
    CollapseSideBar,
}

impl HomeMessage {
    pub fn handle(self, app: &mut Application, id: window::Id) -> Task<Message> {
        match self {
            Self::SearchPreviews(InputMessage::Update(x)) => {
                let page = app.get_home_page(&id).unwrap();
                if x.is_empty() {
                    page.side_bar.previews.clear();
                }
                page.side_bar.search = x;
                Task::none()
            }
            Self::SplitDrag(x) => {
                app.get_home_page(&id).unwrap().side_bar.split =
                    app.windows.get(&id).unwrap().get_size_from_split(x);
                Task::none()
            }
            Self::SearchPreviews(_) => {
                let search = app.get_home_page(&id).unwrap().side_bar.search.clone();
                Task::future(async move {
                    let req = DATA.read().unwrap().to_request();

                    match req
                        .make_request(&format!("preview/search/{}", search), &(), RequestType::Get)
                        .await
                    {
                        Ok(previews) => Message::Window(WindowMessage::Page(
                            id,
                            PageMessage::Home(HomeMessage::SetPreviews(previews)),
                        )),
                        Err(e) => Message::Err(e),
                    }
                })
            }
            Self::SetPreviews(previews) => {
                app.get_home_page(&id).unwrap().side_bar.previews =
                    previews.into_iter().map(|x| x.into()).collect();
                Task::none()
            }
            Self::CollapseSideBar => {
                let page = app.get_home_page(&id).unwrap();
                page.side_bar.is_collapsed = !page.side_bar.is_collapsed;
                Task::none()
            }
            Self::DeleteChat(x) => {
                app.cache.previews.retain(|x| x.id != x.id);

                for window in app.windows.iter_mut() {
                    if let Pages::Home(x) = &mut window.1.page {
                        x.side_bar.previews.retain(|x| x.id != x.id);
                    }
                }

                Task::future(async move {
                    let req = DATA.read().unwrap().to_request();
                    let res: Result<Option<Preview>, String> = req
                        .make_request(&format!("chat/{}", x), &(), RequestType::Delete)
                        .await;
                    match res {
                        Ok(_) => Message::None,
                        Err(e) => Message::Err(e),
                    }
                })
            }
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
                            Ok(x) => {
                                Message::Cache(crate::CacheMessage::AddPreview(PreviewMk::from(x)))
                            }
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
        }
    }
}
