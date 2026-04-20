use iced::advanced::graphics::core::window;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget, tree};
use iced::widget::canvas;
use iced::{Color, Element, Length, Rectangle, Size};
use iced::{Event, Point};
use iced::{Renderer, Vector};
use iced::{alignment, mouse};

pub struct Circle {
    radius: f32,
}

impl Circle {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

pub fn circle(radius: f32) -> Circle {
    Circle::new(radius)
}

impl<Message, Theme> Widget<Message, Theme, Renderer> for Circle {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(self.radius * 2.0, self.radius * 2.0))
    }

    fn update(
        &mut self,
        tree: &mut tree::Tree,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        if shell.is_event_captured() {
            return;
        }

        let is_hovered = if cursor.is_over(layout.bounds()) {
            true
        } else {
            false
        };

        let state = tree.state.downcast_mut::<State>();

        if let Event::Window(window::Event::RedrawRequested(_now)) = event {
            state.is_hovered = is_hovered;
        } else if is_hovered != state.is_hovered {
            state.cache.clear();
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        let state = tree.state.downcast_ref::<State>();

        let color = if state.is_hovered {
            Color::from_rgb(1.0, 0.0, 0.0)
        } else {
            Color::WHITE
        };

        let bounds = layout.bounds();

        let geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            frame.fill_rectangle(Point::ORIGIN, frame.size(), Color::BLACK);

            let center = bounds.center();
            frame.translate(Vector::new(center.x, center.y));
            // let angle = 45.0;
            // frame.rotate(angle * std::f32::consts::PI / 180.0);

            // let rectangle = canvas::Path::rectangle(
            //     Point::new(0.0 - self.radius / 2.0, 0.0 - self.radius / 2.0),
            //     Size::new(self.radius, self.radius),
            // );
            // frame.fill(&rectangle, color);

            // let circle = canvas::Path::circle(frame.center(), self.radius);

            frame.fill_text(canvas::Text {
                content: "▲".to_string(),
                position: Point::ORIGIN,
                color,
                size: bounds.size().width.into(),
                align_x: widget::text::Alignment::Center,
                align_y: alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        });

        renderer.with_translation(
            Vector::new(bounds.x, bounds.y),
            |renderer| {
                use iced::advanced::graphics::geometry::Renderer as _;

                renderer.draw_geometry(geometry);
            },
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let is_mouse_over = cursor.is_over(layout.bounds());

        if is_mouse_over {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

#[derive(Debug, Default)]
struct State {
    is_hovered: bool,
    cache: canvas::Cache,
}

impl<Message, Theme> From<Circle> for Element<'_, Message, Theme, Renderer> {
    fn from(circle: Circle) -> Self {
        Self::new(circle)
    }
}
