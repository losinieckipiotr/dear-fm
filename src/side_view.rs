use iced::Length::Fill;
use iced::widget::{column, container, scrollable, text};
use iced::{Element, Theme, border};

use crate::state::{AppState, Side};
use crate::{Message, table_view};

pub fn side_view(state: &AppState, side: Side) -> Element<'_, Message> {
    column![
        container(
            container(text(format!("{} side", side)))
                .padding(10)
                .style(container_border_style)
                .center_x(Fill),
        )
        .padding([0, 10]),
        scrollable(table_view(state, side))
    ]
    .into()
}

fn container_border_style(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    let style = container::Style::default();
    style.border(border::color(palette.background.strong.color).width(1))
}
