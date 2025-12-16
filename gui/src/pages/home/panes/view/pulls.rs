use crate::{
    Application, DATA, Message,
    font::{BODY_SIZE, HEADER_SIZE, SUB_HEADING_SIZE},
    style,
    subscriptions::{SubMessage, hf_pull::HFPull, ollama_pull::OllamaPull},
};
use iced::{
    Element, Length, Task,
    alignment::Vertical,
    widget::{
        column, container, progress_bar, row,
        rule::horizontal as horizontal_rule,
        scrollable::{self, Scrollbar},
        space::horizontal as horizontal_space,
        text,
    },
};
use ochat_types::providers::{
    hf::{HFModel, HFPullModelStreamResult},
    ollama::{OllamaModelsInfo, OllamaPullModelStreamResult},
};

#[derive(Debug, Clone, Default)]
pub struct PullsView {}

#[derive(Debug, Clone)]
pub enum PullsViewMessage {}

impl PullsViewMessage {
    pub fn handle(self, _app: &mut Application, _id: u32) -> Task<Message> {
        match self {}
    }
}

impl PullsView {
    pub fn view_ollama_pull<'a>(
        id: u32,
        provider_name: String,
        model: &'a OllamaModelsInfo,
        pull: &'a OllamaPull,
    ) -> Element<'a, Message> {
        let sub_heading = |txt: &'static str| text(txt).size(BODY_SIZE).style(style::text::primary);
        let name = text(&pull.model)
            .size(BODY_SIZE + 2)
            .style(style::text::text);

        let provider = text(provider_name)
            .size(BODY_SIZE + 2)
            .style(style::text::translucent::text);

        let model_info = {
            let name = text(&model.name)
                .size(HEADER_SIZE)
                .style(style::text::primary);

            let author = text(&model.author).size(BODY_SIZE);
            let desc = text(&model.description)
                .size(BODY_SIZE)
                .style(style::text::translucent::text);

            column![
                name,
                horizontal_rule(1).style(style::rule::translucent::primary),
                author,
                desc
            ]
            .spacing(5)
        };

        let delete = style::svg_button::danger("delete.svg", HEADER_SIZE)
            .on_press(Message::Subscription(SubMessage::OllamaStopPulling(id)));

        let body: Element<'a, Message> = match &pull.state {
            OllamaPullModelStreamResult::Idle => text("Loading...")
                .style(style::text::translucent::primary)
                .size(BODY_SIZE + 2)
                .into(),
            OllamaPullModelStreamResult::Finished => text("Download Finished!")
                .style(style::text::primary)
                .size(BODY_SIZE + 2)
                .into(),
            OllamaPullModelStreamResult::Err(e) => text(format!("Error - {}", e))
                .style(style::text::danger)
                .size(BODY_SIZE + 2)
                .into(),
            OllamaPullModelStreamResult::Pulling(status) => {
                let mut col = column![].spacing(5);

                if let (Some(total), Some(completed)) = (&status.total, &status.completed) {
                    if total != &0 {
                        let progress = (*completed as f64 / *total as f64) as f32 * 100.0;
                        col = col.push(
                            row![
                                progress_bar(0.0..=100.0, progress)
                                    .length(Length::Fill)
                                    .girth(Length::Fixed(SUB_HEADING_SIZE as f32)),
                                text(format!("{:.2}%", progress))
                                    .style(style::text::primary)
                                    .width(75.0)
                                    .size(SUB_HEADING_SIZE)
                            ]
                            .align_y(Vertical::Center)
                            .spacing(20),
                        );
                    }
                }

                col = col.push(
                    text(&status.status)
                        .style(style::text::translucent::text)
                        .size(BODY_SIZE),
                );

                col.into()
            }
        };

        container(
            column![
                row![
                    column![sub_heading("Name"), name].spacing(5),
                    horizontal_space(),
                    delete,
                ]
                .align_y(Vertical::Center)
                .spacing(20),
                sub_heading("Provider"),
                provider,
                sub_heading("Model"),
                container(model_info)
                    .padding(10)
                    .style(style::container::chat_back),
                sub_heading("Status"),
                container(body)
                    .padding(10)
                    .style(style::container::chat_back),
            ]
            .spacing(5),
        )
        .padding(10)
        .style(style::container::chat)
        .into()
    }

    pub fn view_hf_pull<'a>(id: u32, model: &'a HFModel, pull: &'a HFPull) -> Element<'a, Message> {
        let sub_heading = |txt: &'static str| text(txt).size(BODY_SIZE).style(style::text::primary);
        let name = text(&pull.model)
            .size(BODY_SIZE + 2)
            .style(style::text::text);

        let model_info = {
            let name = text(model.id.split_once("/").unwrap().1)
                .size(HEADER_SIZE)
                .style(style::text::primary);

            let author = text(model.id.split_once("/").unwrap().0).size(BODY_SIZE);

            column![
                name,
                horizontal_rule(1).style(style::rule::translucent::primary),
                author,
            ]
            .spacing(5)
        };

        let delete = style::svg_button::danger("delete.svg", HEADER_SIZE)
            .on_press(Message::Subscription(SubMessage::OllamaStopPulling(id)));

        let body: Element<'a, Message> = match &pull.state {
            HFPullModelStreamResult::Idle => text("Loading...")
                .style(style::text::translucent::primary)
                .size(BODY_SIZE + 2)
                .into(),
            HFPullModelStreamResult::Finished => text("Download Finished!")
                .style(style::text::primary)
                .size(BODY_SIZE + 2)
                .into(),
            HFPullModelStreamResult::Err(e) => text(format!("Error - {}", e))
                .style(style::text::danger)
                .size(BODY_SIZE + 2)
                .into(),
            HFPullModelStreamResult::Pulling(status) => {
                let mut col = column![].spacing(5);

                if let (Some(total), Some(completed)) = (&status.total, &status.completed) {
                    if total != &0 {
                        let progress = (*completed as f64 / *total as f64) as f32 * 100.0;
                        col = col.push(
                            row![
                                progress_bar(0.0..=100.0, progress)
                                    .length(Length::Fill)
                                    .girth(Length::Fixed(SUB_HEADING_SIZE as f32)),
                                text(format!("{:.2}%", progress))
                                    .style(style::text::primary)
                                    .width(75.0)
                                    .size(SUB_HEADING_SIZE)
                            ]
                            .align_y(Vertical::Center)
                            .spacing(20),
                        );
                    }
                }

                col.into()
            }
        };

        container(
            column![
                row![
                    column![sub_heading("Name"), name].spacing(5),
                    horizontal_space(),
                    delete,
                ]
                .align_y(Vertical::Center)
                .spacing(20),
                sub_heading("Model"),
                container(model_info)
                    .padding(10)
                    .style(style::container::chat_back),
                sub_heading("Status"),
                container(body)
                    .padding(10)
                    .style(style::container::chat_back),
            ]
            .spacing(5),
        )
        .padding(10)
        .style(style::container::chat)
        .into()
    }

    pub fn view<'a>(&'a self, app: &'a Application, _id: u32) -> Element<'a, Message> {
        let sub_heading = |txt: &'static str| text(txt).size(BODY_SIZE).style(style::text::primary);
        let mut col = column![sub_heading("Ollama Models")].spacing(20);

        if !app.subscriptions.ollama_pulls.is_empty() {
            col = col.push(
                column(app.subscriptions.ollama_pulls.iter().map(|(k, x)| {
                    Self::view_ollama_pull(
                        k.clone(),
                        DATA.read()
                            .unwrap()
                            .providers
                            .iter()
                            .find(|y| y.id.key().to_string() == x.provider)
                            .unwrap()
                            .name
                            .clone(),
                        app.cache.home_shared.downloads.ollama.get(&k).unwrap(),
                        x,
                    )
                }))
                .spacing(10),
            )
        } else {
            col = col.push(
                text("No Ollama models being pulled")
                    .style(style::text::text)
                    .size(BODY_SIZE + 2),
            )
        }

        col = col.push(sub_heading("Huggingface Models"));

        if !app.subscriptions.hf_pulls.is_empty() {
            col = col.push(
                column(app.subscriptions.hf_pulls.iter().map(|(k, x)| {
                    Self::view_hf_pull(
                        k.clone(),
                        app.cache.home_shared.downloads.hf.get(&k).unwrap(),
                        x,
                    )
                }))
                .spacing(10),
            )
        } else {
            col = col.push(
                text("No Huggingface models being pulled")
                    .style(style::text::text)
                    .size(BODY_SIZE + 2),
            )
        }

        container(
            scrollable::Scrollable::new(col)
                .direction(scrollable::Direction::Vertical(Scrollbar::new()))
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .into()
    }
}
