impl Panes {
    pub fn view<'a>(&'a self, app: &'a ChatApp) -> Element<'a, Message> {
        pane_grid(&self.panes, |pane, state, _is_maximized| {
            let pick = match &app.panes.pick {
                Some(x) => {
                    if pane == x.0 {
                        Some(x.1.clone())
                    } else {
                        None
                    }
                }
                None => None,
            };

            pane_grid::Content::new(match state {
                Pane::Options(x) => add_to_window(
                    app,
                    pane,
                    state.clone(),
                    "Options",
                    pick,
                    app.main_view.options().get(x).unwrap().view(x.clone(), app),
                ),
                Pane::Settings(x) => add_to_window(
                    app,
                    pane,
                    state.clone(),
                    "Settings",
                    pick,
                    app.main_view
                        .settings()
                        .get(x)
                        .unwrap()
                        .view(x.clone(), app),
                ),
                Pane::Tools(x) => add_to_window(
                    app,
                    pane,
                    state.clone(),
                    "Tools",
                    pick,
                    app.main_view.tools().get(x).unwrap().view(x.clone(), app),
                ),
                #[cfg(feature = "voice")]
                Pane::Call => {
                    add_to_window(app, pane, state.clone(), "Call", pick, app.call.view(app))
                }
                Pane::Chat(x) => {
                    if let Some(y) = app.main_view.chats().get(x) {
                        add_to_window(
                            app,
                            pane,
                            state.clone(),
                            "Chat",
                            pick,
                            y.chat_view(app, x.clone()),
                        )
                    } else {
                        text("Please install Ollama to use this app.").into()
                    }
                }
                Pane::Models(x) => add_to_window(
                    app,
                    pane,
                    state.clone(),
                    "Models",
                    pick,
                    app.main_view.models().get(x).unwrap().view(x.clone(), app),
                ),
                Pane::Prompts(x) => add_to_window(
                    app,
                    pane,
                    state.clone(),
                    "Prompts",
                    pick,
                    app.main_view.prompts().get(x).unwrap().view(x.clone(), app),
                ),
                Pane::NoModel => text("Please install Ollama to use this app.").into(),
            })
        })
        .on_drag(|x| Message::Pane(PaneMessage::PaneDragged(x)))
        .on_resize(10, |x| Message::Pane(PaneMessage::PaneResized(x)))
        .width(Length::FillPortion(50))
        .into()
    }
}
