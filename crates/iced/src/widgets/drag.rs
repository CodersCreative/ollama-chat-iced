//! The `Draggable` type and everything needed by it.

use std::any::Any;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use iced::advanced::widget::Tree;
use iced::advanced::{self, Layout, Widget, layout, mouse, overlay, renderer};
use iced::mouse::Button;
use iced::{Element, Size};
use iced::{Rectangle, Vector};

// The global, internally mutable state of the drag and drop system.
#[derive(Debug, Default, Clone)]
pub struct DragAndDrop {
    // The payload being dragged, if any.
    pub dragging: Rc<RefCell<Option<Box<dyn Any>>>>,
}

impl DragAndDrop {
    // Sets the payload to something.
    pub fn set_to<T: 'static>(&self, to: T) {
        self.dragging.replace(Some(Box::new(to)));
    }

    // Clears the payload, setting it to None.
    pub fn clear(&self) {
        self.dragging.replace(None);
    }

    // Checks if the user is dragging something.
    pub fn has_some(&self) -> bool {
        self.dragging.borrow().is_some()
    }
}

/// A helper for creating Draggables. Needs the drag and drop state, a payload, and inner content.
#[must_use = "This Element is being created but is not used anywhere. Make sure it has a payload"]
pub fn drag<'a, Message, Theme, Renderer, Payload>(
    id: String,
    dragging: DragAndDrop,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Draggable<'a, Message, Theme, Renderer, Payload>
where
    Message: Clone,
    Payload: Clone,
    Renderer: iced::advanced::Renderer,
{
    Draggable {
        dragging,
        content: content.into(),
        on_pickup: None,
        payload: None,
        id,
    }
}

/// An Element that can be dragged across the screen.
pub struct Draggable<'a, Message: Clone, Theme, Renderer, Payload> {
    pub id: String,
    /// A reference to the global drag and drop state
    pub dragging: DragAndDrop,
    /// The visual elements of the draggable
    pub content: Element<'a, Message, Theme, Renderer>,
    /// Send a message on pick up?
    pub on_pickup: Option<Box<dyn Fn(Payload) -> Message>>,
    /// The payload of the draggable. This is intermediary data used by drop zones and messages to modify your program's state.
    pub payload: Option<Payload>,
}

impl<
    'a,
    Message: Clone + 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
    Payload: Clone + 'static,
> From<Draggable<'a, Message, Theme, Renderer, Payload>> for Element<'a, Message, Theme, Renderer>
{
    fn from(value: Draggable<'a, Message, Theme, Renderer, Payload>) -> Self {
        Self::new(value)
    }
}

impl<Message: Clone, Theme, Renderer: iced::advanced::Renderer, Payload: Clone>
    Draggable<'_, Message, Theme, Renderer, Payload>
{
    /// Send a message when object begins being dragged.
    #[must_use = "This Element is being created but is not used anywhere"]
    pub fn on_pickup<F: (Fn(Payload) -> Message) + 'static>(self, fun: F) -> Self {
        Self {
            on_pickup: Some(Box::new(fun)),
            ..self
        }
    }

    #[must_use = "This draggable is given a payload but is not used anywhere"]
    pub fn payload(self, payload: Payload) -> Self {
        Self {
            payload: Some(payload),
            ..self
        }
    }
}

impl<Message: Clone, Theme, Renderer: iced::advanced::Renderer, Payload: Clone + 'static>
    Widget<Message, Theme, Renderer> for Draggable<'_, Message, Theme, Renderer, Payload>
{
    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(State::default())
    }
    fn size(&self) -> iced::Size<iced::Length> {
        self.content.as_widget().size()
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![advanced::widget::Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        let state = tree.state.downcast_mut::<State>();
        if state.prev_id != self.id {
            *state = State::default();
        }
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
        let state = tree.state.downcast_mut::<State>();
        if state.dragging {
            return Some(overlay::Element::new(Box::new(Overlay {
                content: &mut self.content,
                tree: &mut tree.children[0],
                bounds: state.overlay_bounds,
            })));
        }
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
        let size = self.size();
        layout::padded(limits, size.width, size.height, 0, |limits| {
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
        let state = tree.state.downcast_ref::<State>();
        let content_layout = layout.children().next().unwrap();

        if !state.dragging {
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

        if shell.is_event_captured() {
            return;
        }

        // TODO
        // Add dragging. Clicking should set the dragging state to true. Releasing should set it to false.
        match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(Button::Left)) => {
                let bounds = layout.bounds();
                if cursor.is_over(bounds) {
                    let state = state.state.downcast_mut::<State>();
                    state.overlay_bounds.width = bounds.width;
                    state.overlay_bounds.height = bounds.height;

                    state.is_pressed = true;

                    shell.capture_event();
                }
            }
            iced::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                let bounds = layout.bounds();
                let state = state.state.downcast_mut::<State>();
                state.overlay_bounds.x = position.x - state.overlay_bounds.width / 2.0;
                state.overlay_bounds.y = position.y - state.overlay_bounds.height / 2.0;
                if state.is_pressed && (cursor.is_over(bounds) || state.dragging) {
                    if !state.dragging {
                        state.dragging = true;
                        if let Some(payload) = &self.payload {
                            self.dragging.set_to(payload.clone());
                        }
                        if let (Some(on_pickup), Some(payload)) = (&self.on_pickup, &self.payload) {
                            shell.publish(on_pickup(payload.clone()));
                        }
                    }

                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(Button::Left)) => {
                let state = state.state.downcast_mut::<State>();
                state.dragging = false;
                state.is_pressed = false;
                shell.request_redraw();
            }
            _ => {}
        }
    }
}

#[derive(Default)]
struct State {
    is_pressed: bool,
    dragging: bool,
    overlay_bounds: Rectangle,
    prev_id: String,
}
struct Overlay<'a, 'b, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    content: &'b mut Element<'a, Message, Theme, Renderer>,
    tree: &'b mut advanced::widget::Tree,
    bounds: Rectangle,
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'_, '_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, _bounds: Size) -> layout::Node {
        Widget::<Message, Theme, Renderer>::layout(
            self.content.as_widget_mut(),
            self.tree,
            renderer,
            &layout::Limits::new(Size::ZERO, self.bounds.size()),
        )
        .move_to(self.bounds.position())
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
    ) {
        Widget::<Message, Theme, Renderer>::draw(
            self.content.as_widget(),
            self.tree,
            renderer,
            theme,
            inherited_style,
            layout,
            cursor_position,
            &Rectangle::with_size(Size::INFINITE),
        );
    }
}
