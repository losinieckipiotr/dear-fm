use crate::Application;
use crate::message::Message;
use crate::side_view::side_view;
use crate::state::Side;
use iced::Element;
use iced::widget::{row, rule, text};

pub fn view(app: &Application) -> Element<'_, Message> {
    if !app.loaded {
        return text("loading...").into();
    }

    row![
        side_view(&app.state, Side::Left),
        rule::vertical(1),
        side_view(&app.state, Side::Right),
    ]
    .into()
}
