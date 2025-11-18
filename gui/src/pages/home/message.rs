use iced::{Task, window};
use ochat_types::chats::previews::Preview;

use crate::{
    Application, DATA, Message,
    data::RequestType,
    pages::{PageMessage, Pages, home::HomePaneType},
    windows::message::WindowMessage,
};

#[derive(Debug, Clone)]
pub enum HomePickingType {
    ReplaceChat(String),
    OpenPane(HomePaneType),
}

#[derive(Debug, Clone)]
pub enum HomeMessage {
    PanePick(HomePickingType),
    ChangeSearchPreviews(String),
    SubmitSearchPreviews,
    SetPreviews(Vec<Preview>),
    NewChat,
    DeleteChat(String),
    CollapseSideBar,
}

impl HomeMessage {
    pub fn handle(self, app: &mut Application, id: window::Id) -> Task<Message> {
        let Pages::Home(ref mut page) = app.windows.get_mut(&id).unwrap().page else {
            return Task::none();
        };

        match self {
            Self::PanePick(x) => {
                // TODO Handle pane creation logic and add the rest of the panes
                page.panes.pick = Some(x);
                Task::none()
            }
            Self::ChangeSearchPreviews(x) => {
                if x.is_empty() {
                    page.side_bar.previews.clear();
                }
                page.side_bar.search = x;
                Task::none()
            }
            Self::SubmitSearchPreviews => {
                let search = page.side_bar.search.clone();
                Task::future(async move {
                    let req = DATA.read().unwrap().to_request();

                    let previews = req
                        .make_request(&format!("preview/search/{}", search), &(), RequestType::Get)
                        .await
                        .unwrap_or_default();

                    Message::Window(WindowMessage::Page(
                        id,
                        PageMessage::Home(HomeMessage::SetPreviews(previews)),
                    ))
                })
            }
            Self::SetPreviews(previews) => {
                page.side_bar.previews = previews.into_iter().map(|x| x.into()).collect();
                Task::none()
            }
            Self::CollapseSideBar => {
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
                    let _: Result<Option<Preview>, String> = req
                        .make_request(&format!("chat/{}", x), &(), RequestType::Delete)
                        .await;
                    Message::None
                })
            }
            // TODO finish all other cases
            _ => Task::none(),
        }
    }
}
