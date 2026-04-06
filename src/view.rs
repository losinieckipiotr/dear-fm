use crate::Application;
use crate::message::Message;
use crate::side_view::side_view;
use crate::state::Side;
use iced::{Element, widget};

pub fn view(app: &Application) -> Element<'_, Message> {
    if !app.loaded {
        return widget::text("loading...").into();
    }

    widget::row![
        side_view(&app.state, Side::Left),
        widget::rule::vertical(1),
        side_view(&app.state, Side::Right),
    ]
    .into()
}
