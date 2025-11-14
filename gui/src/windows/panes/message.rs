#[derive(Debug, Clone)]
pub enum PaneMessage {
    Clicked(pane_grid::Pane, Pane),
    Pick(pane_grid::Pane, Pane),
    UnPick,
    Close(pane_grid::Pane),
    PaneDragged(pane_grid::DragEvent),
    PaneResized(pane_grid::ResizeEvent),
    Split(pane_grid::Axis, pane_grid::Pane, Pane),
    Replace(pane_grid::Pane, Pane),
}
impl PaneMessage {
    pub fn handle(&self, app: &mut ChatApp) -> Task<Message> {
        match self {
            Self::Clicked(pane, state) => {
                app.panes.focus = Some(*pane);
                if let Pane::Chat(x) = state {
                    app.panes.last_chat = x.clone();
                }
                Task::none()
            }
            Self::Close(pane) => {
                if app.panes.created > 1 {
                    if let Some((_, sibling)) = app.panes.panes.close(*pane) {
                        app.panes.focus = Some(sibling);
                        #[cfg(feature = "voice")]
                        if let Some(call) = app.panes.call {
                            if call == *pane {
                                app.panes.call = None;
                            }
                        }
                    }
                }
                Task::none()
            }
            Self::PaneDragged(pane_grid::DragEvent::Dropped { pane, target }) => {
                app.panes.panes.drop(*pane, *target);
                Task::none()
            }
            Self::PaneDragged(_) => Task::none(),
            Self::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                app.panes.panes.resize(*split, *ratio);
                Task::none()
            }
            Self::Replace(grid_pane, pane) => {
                let value = match pane {
                    Pane::Options(_) => {
                        Pane::new_options(app, app.logic.models.first().unwrap().clone())
                    }
                    Pane::Settings(_) => Pane::new_settings(app),
                    Pane::Tools(_) => Pane::new_tools(app),
                    Pane::Chat(x) => {
                        let id = Id::new();
                        app.main_view.add_to_chats(
                            id.clone(),
                            app.main_view.chats().get(x).unwrap().clone(),
                        );
                        Pane::Chat(id)
                    }
                    Pane::Models(_) => Pane::new_models(app),
                    Pane::Prompts(_) => Pane::new_prompts(app),
                    #[cfg(feature = "voice")]
                    Pane::Call => Pane::Call,
                    _ => Pane::NoModel,
                };

                let result =
                    app.panes
                        .panes
                        .split(pane_grid::Axis::Vertical, *grid_pane, value.clone());

                if let Some((pane, _)) = result {
                    app.panes.focus = Some(pane);
                }

                app.panes.pick = None;

                if let Pane::Chat(x) = pane {
                    app.panes.last_chat = *x;
                }
                app.panes.panes.close(*grid_pane);

                Task::none()
            }
            Self::Pick(grid_pane, pane) => {
                Panes::new_window(app, *grid_pane, pane.clone());
                Task::none()
            }
            Self::UnPick => {
                app.panes.pick = None;
                Task::none()
            }
            Self::Split(axis, og, pane) => {
                let result = app.panes.panes.split(*axis, *og, pane.clone());

                if let Some((p, _)) = result {
                    app.panes.focus = Some(p);
                    #[cfg(feature = "voice")]
                    if let Pane::Call = pane {
                        app.panes.call = Some(p);
                    }
                }

                app.panes.pick = None;
                if let Pane::Chat(x) = pane {
                    app.panes.last_chat = *x;
                }

                app.panes.created += 1;
                Task::none()
            }
        }
    }
}
