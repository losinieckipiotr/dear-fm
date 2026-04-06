use chrono::{DateTime, Local};
use humansize::{DECIMAL, format_size};
use iced::Length::Fill;

use iced::widget::{self, button, container, mouse_area, opaque, text};
use iced::{Background, Element, Padding, Theme};

use crate::Message;
use crate::files::FileRecord;
use crate::state::{AppState, Side};

fn header_container_style(
    hover: bool,
    theme: &Theme,
) -> widget::container::Style {
    widget::container::Style {
        background: match hover {
            true => None,
            false => Some(Background::Color(
                theme.extended_palette().background.strong.color,
            )),
        },
        ..widget::container::Style::default()
    }
}

fn header<'a>(
    state: &'a AppState,
    title: &'a str,
    idx: usize,
) -> Element<'a, Message> {
    opaque(
        mouse_area(
            widget::container(widget::text(title))
                .padding(Padding::from([0, 5]))
                .align_left(Fill)
                .style(|theme| {
                    let hover = state.header_hover.iter().any(|i| *i);
                    header_container_style(hover, theme)
                }),
        )
        .on_enter(Message::HeaderHover(idx, true))
        .on_exit(Message::HeaderHover(idx, false))
        .on_press(Message::TestClick),
    )
    .into()
}

fn name_view(
    side: Side,
    idx: usize,
    file: &FileRecord,
) -> Element<'_, Message> {
    button(text(file.file_name.as_str()))
        .on_press(Message::OpenFileOrDir(side, idx))
        .into()
}

fn name_column_view(
    state: &AppState,
    side: Side,
) -> widget::Column<'_, Message> {
    let col = widget::Column::new().push(header(state, "Name", 0));

    let names: Vec<Element<'_, Message>> = state
        .get_window_files(side)
        .iter()
        .enumerate()
        .map(|(idx, file)| name_view(side, idx, file))
        .collect();

    col.extend(names)
}

fn size_view(file: &FileRecord) -> Element<'_, Message> {
    let view = if file.is_go_back_record {
        text("")
    } else {
        let size = file.size;
        let is_file = file.is_file;

        let formatted_size: String =
            format_size(size, DECIMAL.decimal_places(1));
        if is_file {
            text(formatted_size)
        } else {
            text("--")
        }
    };

    view.into()
}

fn size_column_view(
    state: &AppState,
    side: Side,
) -> widget::Column<'_, Message> {
    let col = widget::Column::new().push(header(state, "Size", 1));

    let names: Vec<Element<'_, Message>> = state
        .get_window_files(side)
        .iter()
        .enumerate()
        .map(|(_idx, file)| size_view(file))
        .collect();

    col.extend(names)
}

fn modified_view(file: &FileRecord) -> Element<'_, Message> {
    let view = if file.is_go_back_record {
        text("")
    } else {
        let modified = file.modified;
        let datetime: DateTime<Local> = modified.into();

        text(datetime.format("%d %b %Y at %H:%M").to_string())
    };

    opaque(mouse_area(view).on_press(Message::TestClick)).into()
}

fn modified_column_view(
    state: &AppState,
    side: Side,
) -> widget::Column<'_, Message> {
    let col = widget::Column::new().push(header(state, "Modified", 2));

    let names: Vec<Element<'_, Message>> = state
        .get_window_files(side)
        .iter()
        .enumerate()
        .map(|(_idx, file)| modified_view(file))
        .collect();

    col.extend(names)
}

pub fn table_view(state: &AppState, side: Side) -> Element<'_, Message> {
    let name_col = name_column_view(state, side);
    let size_col = size_column_view(state, side);
    let modified_col = modified_column_view(state, side);

    let table = widget::row![name_col, size_col, modified_col];
    container(table).padding(Padding::from([0, 10])).into()
}
