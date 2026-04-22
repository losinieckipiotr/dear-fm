use std::path::PathBuf;

use crate::circle::circle;
use crate::message::Message;
use crate::state::{AppState, Side};
use crate::{Application, table_view::table_view};
use iced::Length::Fill;
use iced::widget::{Column, Row, button, container, rule, text};
use iced::{Element, Theme, alignment, border};

pub fn view(app: &Application) -> Element<'_, Message> {
    let mut col = Column::new();

    col = col.push(circle(40.0));

    col = col.push(
        Row::new()
            .push(side_view(&app.state, Side::Left))
            .push(rule::vertical(1))
            .push(side_view(&app.state, Side::Right)),
    );

    if app.loading {
        col = col.push(
            text("loading...")
                .align_x(alignment::Horizontal::Center)
                .width(Fill),
        );
    }

    col.into()
}

fn side_view(state: &AppState, side: Side) -> Element<'_, Message> {
    let path = state.get_path(side);
    let buf = PathBuf::from(path);

    let buttons: Vec<Element<'_, Message>> = buf
        .iter()
        .enumerate()
        .map(|(i, part)| {
            let path_to_open = PathBuf::from_iter(buf.iter().take(i + 1));

            button(
                text(part.display().to_string()).wrapping(text::Wrapping::None),
            )
            .clip(true)
            .style(button::secondary)
            .on_press(Message::PathButtonClick(side, path_to_open))
            .into()
        })
        .collect();

    let row = Row::new().extend(buttons).spacing(5);

    Column::new()
        .push(
            container(
                container(row)
                    .padding(10)
                    .style(|theme: &Theme| container::Style {
                        border: border::color(
                            theme.extended_palette().background.neutral.color,
                        )
                        .width(1),
                        ..Default::default()
                    })
                    .align_left(Fill),
            )
            .padding([0, 0]),
        )
        .push(table_view(state, side))
        .into()
}
