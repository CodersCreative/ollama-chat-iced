use iced::{Task, widget::pane_grid, window};

use crate::{
    Application, Message,
    pages::{
        Pages,
        home::{
            HomePage,
            message::HomePickingType,
            panes::view::{
                models::ModelsView, options::OptionsView, prompts::PromptsView, pulls::PullsView,
                settings::SettingsView,
            },
        },
    },
    windows::message::WindowMessage,
};

pub mod data;
pub mod view;

#[derive(Debug, Clone)]
pub enum HomePaneType {
    Chat,
    Pulls,
    Models,
    Prompts,
    Options,
    Settings,
    Tools,
}

impl HomePaneType {
    pub fn new(&self, app: &mut Application) -> HomePaneTypeWithId {
        app.view_data.counter += 1;
        let count = app.view_data.counter;

        match self {
            Self::Models => {
                app.view_data
                    .home
                    .models
                    .insert(count, ModelsView::default());
                HomePaneTypeWithId::Models(count)
            }
            Self::Prompts => {
                app.view_data
                    .home
                    .prompts
                    .insert(count, PromptsView::default());
                HomePaneTypeWithId::Prompts(count)
            }
            Self::Settings => {
                let mut settings = SettingsView::default();
                settings.instance_url = app.cache.client_settings.instance_url.clone();
                app.view_data.home.settings.insert(count, settings);
                HomePaneTypeWithId::Settings(count)
            }
            Self::Options => {
                app.view_data
                    .home
                    .options
                    .insert(count, OptionsView::default());
                HomePaneTypeWithId::Options(count)
            }
            Self::Pulls => {
                app.view_data.home.pulls.insert(count, PullsView::default());
                HomePaneTypeWithId::Pulls(count)
            }
            _ => HomePaneTypeWithId::Chat(count),
        }
    }
}

#[derive(Debug, Clone)]
pub enum HomePaneTypeWithId {
    Chat(u32),
    Pulls(u32),
    Models(u32),
    Prompts(u32),
    Options(u32),
    Settings(u32),
    Tools(u32),
}

impl Into<HomePaneType> for &HomePaneTypeWithId {
    fn into(self) -> HomePaneType {
        match self {
            HomePaneTypeWithId::Chat(_) => HomePaneType::Chat,
            HomePaneTypeWithId::Pulls(_) => HomePaneType::Pulls,
            HomePaneTypeWithId::Models(_) => HomePaneType::Models,
            HomePaneTypeWithId::Prompts(_) => HomePaneType::Prompts,
            HomePaneTypeWithId::Options(_) => HomePaneType::Options,
            HomePaneTypeWithId::Settings(_) => HomePaneType::Settings,
            HomePaneTypeWithId::Tools(_) => HomePaneType::Tools,
        }
    }
}

impl Into<HomePaneType> for HomePaneTypeWithId {
    fn into(self) -> HomePaneType {
        (&self).into()
    }
}

#[derive(Debug, Clone)]
pub struct HomePanes {
    pub focus: Option<pane_grid::Pane>,
    pub panes: pane_grid::State<HomePaneTypeWithId>,
    pub pick: Option<HomePickingType>,
}

impl HomePanes {
    pub fn new(pane: HomePaneTypeWithId) -> Self {
        let (panes, _) = pane_grid::State::new(pane);
        let (focus, _) = panes.panes.iter().last().unwrap();

        Self {
            focus: Some(focus.clone()),
            panes,
            pick: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PaneMessage {
    Close(pane_grid::Pane),
    NewWindow(pane_grid::Pane),
    Clicked(pane_grid::Pane),
    Dragged(pane_grid::DragEvent),
    Resized(pane_grid::ResizeEvent),
    Split(pane_grid::Axis, pane_grid::Pane, HomePaneType),
    Replace(pane_grid::Pane, HomePaneType),
    ReplaceChat(pane_grid::Pane, String),
    Pick(HomePickingType),
    UnPick,
}

impl PaneMessage {
    pub fn handle(self, app: &mut Application, id: window::Id) -> Task<Message> {
        match self {
            Self::Pick(x) => {
                match x {
                    HomePickingType::OpenPane(x) if !app.cache.client_settings.use_panes => {
                        return Task::done(Message::Window(WindowMessage::Page(
                            id,
                            crate::pages::PageMessage::Home(super::message::HomeMessage::Pane(
                                PaneMessage::Replace(
                                    app.get_home_page(&id)
                                        .unwrap()
                                        .panes
                                        .panes
                                        .panes
                                        .first_key_value()
                                        .unwrap()
                                        .0
                                        .clone(),
                                    x,
                                ),
                            )),
                        )));
                    }
                    _ => app.get_home_page(&id).unwrap().panes.pick = Some(x),
                }

                Task::none()
            }
            Self::UnPick => {
                app.get_home_page(&id).unwrap().panes.pick = None;
                Task::none()
            }
            Self::Dragged(pane_grid::DragEvent::Dropped { pane, target }) => {
                app.get_home_page(&id)
                    .unwrap()
                    .panes
                    .panes
                    .drop(pane, target);
                Task::none()
            }
            Self::Dragged(_) => Task::none(),
            Self::Resized(pane_grid::ResizeEvent { split, ratio }) => {
                app.get_home_page(&id)
                    .unwrap()
                    .panes
                    .panes
                    .resize(split, ratio);
                Task::none()
            }
            Self::Clicked(pane) => {
                app.get_home_page(&id).unwrap().panes.focus = Some(pane);
                Task::none()
            }
            Self::Close(pane) => {
                let page = app.get_home_page(&id).unwrap();
                if page.panes.panes.len() <= 1 {
                    return Task::none();
                }

                // TODO remove pane from view_data
                if let Some((_, sibling)) = page.panes.panes.close(pane) {
                    page.panes.focus = Some(sibling);
                }

                Task::none()
            }
            Self::Replace(pane, pane_type) => {
                let value = pane_type.new(app);
                let page = app.get_home_page(&id).unwrap();

                // TODO remove pane from view_data
                let _ = page.panes.panes.panes.insert(pane.clone(), value);

                page.panes.pick = None;
                page.panes.focus = Some(pane);

                Task::none()
            }
            Self::NewWindow(pane) => {
                let page = app.get_home_page(&id).unwrap();

                if page.panes.panes.len() <= 1 {
                    return Task::none();
                }

                if let Some((value, sibling)) = page.panes.panes.close(pane) {
                    page.panes.focus = Some(sibling);

                    let mut page = HomePage::new();
                    page.panes = HomePanes::new(value);
                    app.view_data.page_stack.push(Pages::Home(page));
                } else {
                    return Task::none();
                }

                Task::done(Message::Window(WindowMessage::OpenWindow))
            }

            Self::Split(axis, pane, pane_type) => {
                let value = pane_type.new(app);
                let page = app.get_home_page(&id).unwrap();
                let result = page.panes.panes.split(axis, pane, value);

                if let Some((p, _)) = result {
                    page.panes.focus = Some(p);
                }

                page.panes.pick = None;

                Task::none()
            }
            PaneMessage::ReplaceChat(_pane, _chat_id) => Task::none(),
        }
    }
}
