//! The `DropZone` type and everything needed by it.

use iced::advanced::widget::Tree;
use iced::advanced::{self, Layout, Widget, layout, mouse, overlay, renderer};
use iced::mouse::Button;
use iced::{Element, Length, Padding, Rectangle, Size, Vector};

use crate::widgets::drag::DragAndDrop;

/// A helper for creating `DropZones`. Needs the drag and drop state as well as inner content. Make sure to give it an `on_drop` message.
#[must_use = "This zone is not used anywhere. Make sure to give it an `on_drop` message."]
pub fn drop_zone<'a, Payload, Message, Theme, Renderer>(
    dragging: DragAndDrop,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> DropZone<'a, Message, Payload, Theme, Renderer>
where
    Message: Clone,
    Payload: Clone,
    Renderer: iced::advanced::Renderer,
{
    DropZone {
        width: Length::Shrink,
        height: Length::Shrink,
        padding: Padding::new(0.0),
        content: content.into(),
        on_drop: None,
        dragging,
    }
}

pub struct DropZone<
    'a,
    Message: Clone,
    Payload: Clone = (),
    Theme = iced::widget::Theme,
    Renderer = iced::widget::Renderer,
> {
    pub width: Length,
    pub height: Length,
    pub padding: Padding,
    pub content: Element<'a, Message, Theme, Renderer>,
    /// Send a message when something is dropped?
    pub on_drop: Option<OnDrop<'a, Message, Payload>>,
    /// The global drag and drop system
    pub dragging: DragAndDrop,
}

impl<'a, Message: Clone, Theme, Renderer, Payload: Clone>
    DropZone<'a, Message, Payload, Theme, Renderer>
{
    /// Sends a message when something is dropped in this zone.
    #[must_use = "This Element is not used anywhere. Make sure it is retuned by your view function in some container."]
    pub fn on_drop(self, fun: impl Fn(Payload) -> Message + 'a) -> Self {
        Self {
            on_drop: Some(OnDrop::Closure(Box::new(fun))),
            ..self
        }
    }
}

impl<
    'a,
    Message: Clone + 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
    Payload: Clone + 'static,
> From<DropZone<'a, Message, Payload, Theme, Renderer>> for Element<'a, Message, Theme, Renderer>
{
    fn from(value: DropZone<'a, Message, Payload, Theme, Renderer>) -> Self {
        Self::new(value)
    }
}

pub enum OnDrop<'a, Message: Clone, Payload: Clone> {
    Direct(Message),
    Closure(Box<dyn Fn(Payload) -> Message + 'a>),
}

impl<Message: Clone, Theme, Renderer: iced::advanced::Renderer, Payload: Clone + 'static>
    Widget<Message, Theme, Renderer> for DropZone<'_, Message, Payload, Theme, Renderer>
{
    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(State::default())
    }
    fn size(&self) -> iced::Size<iced::Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![advanced::widget::Tree::new(&self.content)]
    }

    #[allow(clippy::semicolon_if_nothing_returned)]
    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content))
    }

    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        advanced::widget::tree::Tag::of::<State>()
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        layout::padded(limits, self.width, self.height, self.padding, |limits| {
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits)
        })
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let content_layout = layout.children().next().unwrap();

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: style.text_color,
            },
            content_layout,
            cursor,
            viewport,
        );
    }

    fn update(
        &mut self,
        state: &mut Tree,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut state.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        // ALLOW
        // We're allowing this because, in the future, there are other things I'd like to
        // detect here
        #[allow(clippy::single_match_else)]
        match event {
            iced::Event::Mouse(mouse::Event::ButtonReleased(Button::Left)) => {
                let state = state.state.downcast_mut::<State>();

                let bounds = layout.bounds();

                if cursor.is_over(bounds) {
                    if let Some(on_drop) = &self.on_drop {
                        if let Some(payload) = self.dragging.dragging.take() {
                            if let Ok(payload) = payload.downcast::<Payload>() {
                                shell.publish(on_drop.get(*payload));
                                shell.capture_event();
                            }
                        }
                    }
                }
                state.dragging_on = false;
            }
            _ => {
                let state = state.state.downcast_mut::<State>();
                let bounds = layout.bounds();
                state.dragging_on = cursor.is_over(bounds) && self.dragging.has_some();
            }
        }
    }
}

#[derive(Default)]
struct State {
    dragging_on: bool,
}

impl<Message: Clone, Payload: Clone> OnDrop<'_, Message, Payload> {
    fn get(&self, payload: Payload) -> Message {
        match self {
            Self::Direct(message) => message.clone(),
            Self::Closure(f) => f(payload),
        }
    }
}
