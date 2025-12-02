use iced::{
    Element, Length, Task,
    application::Title,
    widget::{
        column, container, horizontal_rule, horizontal_space, row,
        scrollable::{self, Scrollbar},
        text, text_input,
    },
};
use ochat_types::prompts::Prompt;

use crate::{
    Application, InputMessage, Message,
    font::{HEADER_SIZE, SUB_HEADING_SIZE},
    pages::home::panes::{data::PromptsData, view::HomePaneViewMessage},
    style,
};

#[derive(Debug, Clone)]
pub struct PromptsView {
    pub search: String,
    pub expanded: Vec<String>,
    pub editing: Vec<String>,
    pub prompts: PromptsData,
}

impl Default for PromptsView {
    fn default() -> Self {
        Self {
            search: String::default(),
            expanded: Vec::new(),
            editing: Vec::new(),
            prompts: PromptsData::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PromptsViewMessage {
    Search(InputMessage),
    SetPrompts(PromptsData),
    Expand(String),
    Edit(String),
    AddNew,
    FinishEditing(usize),
    Upload,
}

impl PromptsViewMessage {
    pub fn handle(self, app: &mut Application, id: u32) -> Task<Message> {
        match self {
            Self::Search(InputMessage::Update(x)) => {
                let view = app.get_prompts_view(&id).unwrap();

                if x.is_empty() {
                    view.prompts.0.clear();
                }

                view.search = x;
                Task::none()
            }
            Self::Search(_) => {
                let search = app.get_models_view(&id).unwrap().search.clone();
                Task::future(async move {
                    Message::HomePaneView(HomePaneViewMessage::Prompts(
                        id,
                        PromptsViewMessage::SetPrompts(
                            PromptsData::get_prompts(Some(search)).await,
                        ),
                    ))
                })
            }
            Self::SetPrompts(x) => {
                app.get_prompts_view(&id).unwrap().prompts = x;
                Task::none()
            }
            Self::Expand(x) => {
                let view = app.get_prompts_view(&id).unwrap();
                if view.expanded.contains(&x) {
                    view.expanded.retain(|y| y != &x);
                } else {
                    view.expanded.push(x);
                }

                Task::none()
            }
            Self::Edit(x) => {
                let view = app.get_prompts_view(&id).unwrap();
                if view.editing.contains(&x) {
                    view.editing.retain(|y| y != &x);
                } else {
                    view.editing.push(x);
                }

                Task::none()
            }
            _ => Task::none(),
        }
    }
}

impl PromptsView {
    pub fn view_prompt<'a>(
        id: u32,
        prompt: &'a Prompt,
        expanded: bool,
        editing: bool,
    ) -> Element<'a, Message> {
        // let sub_heading = |txt: &'static str| text(txt).size(BODY_SIZE).style(style::text::text);

        let edit =
            style::svg_button::primary(if editing { "close.svg" } else { "edit.svg" }, HEADER_SIZE)
                .on_press(Message::HomePaneView(HomePaneViewMessage::Prompts(
                    id,
                    PromptsViewMessage::Edit(prompt.id.key().to_string()),
                )));

        let expand = style::svg_button::primary(
            if expanded {
                "arrow_drop_up.svg"
            } else {
                "arrow_drop_down.svg"
            },
            HEADER_SIZE,
        )
        .on_press(Message::HomePaneView(HomePaneViewMessage::Prompts(
            id,
            PromptsViewMessage::Expand(prompt.id.key().to_string()),
        )));

        let col = if editing {
            let title = text(&prompt.title)
                .size(HEADER_SIZE)
                .style(style::text::primary);

            let mut col = column![
                row![title, horizontal_space(), edit],
                horizontal_rule(1).style(style::rule::translucent::primary),
            ]
            .spacing(10);

            col
        } else if expanded {
            let title = text(&prompt.title)
                .size(HEADER_SIZE)
                .style(style::text::primary);

            let mut col = column![
                row![title, horizontal_space(), expand],
                horizontal_rule(1).style(style::rule::translucent::primary),
            ]
            .spacing(10);

            col
        } else {
            let title = text(&prompt.title)
                .size(HEADER_SIZE)
                .style(style::text::primary);

            column![row![title, horizontal_space(), edit, expand],].spacing(10)
        };

        container(col).into()
    }

    pub fn view<'a>(&'a self, app: &'a Application, id: u32) -> Element<'a, Message> {
        let search = style::svg_input::primary(
            Some(String::from("search.svg")),
            text_input("Search prompts...", &self.search)
                .on_input(move |x| {
                    Message::HomePaneView(HomePaneViewMessage::Prompts(
                        id,
                        PromptsViewMessage::Search(InputMessage::Update(x)),
                    ))
                })
                .on_submit(Message::HomePaneView(HomePaneViewMessage::Prompts(
                    id,
                    PromptsViewMessage::Search(InputMessage::Submit),
                ))),
            SUB_HEADING_SIZE,
        );

        let add = style::svg_button::primary("add.svg", SUB_HEADING_SIZE).on_press(
            Message::HomePaneView(HomePaneViewMessage::Prompts(id, PromptsViewMessage::AddNew)),
        );

        let prompts = scrollable::Scrollable::new(
            column(
                if self.search.is_empty() || self.prompts.0.is_empty() {
                    &app.cache.home_shared.prompts.0
                } else {
                    &self.prompts.0
                }
                .iter()
                .map(|x| {
                    Self::view_prompt(
                        id.clone(),
                        x,
                        self.expanded.contains(&x.id.key().to_string()),
                        self.editing.contains(&x.id.key().to_string()),
                    )
                }),
            )
            .spacing(10),
        )
        .direction(scrollable::Direction::Vertical(Scrollbar::new()))
        .width(Length::Fill)
        .height(Length::Fill);

        container(column![row![search, add], prompts].spacing(10)).into()
    }
}
