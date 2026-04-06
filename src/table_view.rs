use chrono::{DateTime, Local};
use humansize::{DECIMAL, format_size};
use iced::Length::Fill;

use iced::widget::{
    Column, container, mouse_area, opaque, row, scrollable, text,
};
use iced::{Background, Element, Padding, Theme};

use crate::Message;
use crate::files::FileRecord;
use crate::state::{AppState, Side};

fn header_container_style(hover: bool, theme: &Theme) -> container::Style {
    container::Style {
        background: match hover {
            true => None,
            false => Some(Background::Color(
                theme.extended_palette().background.strong.color,
            )),
        },
        ..Default::default()
    }
}

fn header<'a>(
    state: &'a AppState,
    title: &'a str,
    idx: usize,
) -> Element<'a, Message> {
    opaque(
        mouse_area(
            container(text(title))
                .padding(Padding::from([0, 5]))
                .align_left(Fill)
                .style(|theme| {
                    let hover = state.header_hover.iter().any(|i| *i);
                    header_container_style(hover, theme)
                }),
        )
        .on_enter(Message::HeaderHover(idx, true))
        .on_exit(Message::HeaderHover(idx, false)),
    )
    .into()
}
fn table_text_item(
    side: Side,
    idx: usize,
    selected_idx: Option<usize>,
    data: String,
) -> Element<'static, Message> {
    let is_selected = match selected_idx {
        Some(selected_idx) => selected_idx == idx,
        None => false,
    };

    opaque(
        mouse_area(
            container(text(data).wrapping(text::Wrapping::None))
                .width(Fill)
                .style(move |theme: &Theme| container::Style {
                    background: match is_selected {
                        true => Some(Background::Color(
                            theme.extended_palette().primary.base.color,
                        )),
                        false => None,
                    },
                    ..Default::default()
                })
                .clip(true),
        )
        .on_press(Message::SelectIdx(side, idx))
        .on_double_click(Message::OpenFileOrDir(side, idx)),
    )
    .into()
}

fn name_view(
    side: Side,
    idx: usize,
    selected_idx: Option<usize>,
    file: &FileRecord,
) -> Element<'_, Message> {
    table_text_item(side, idx, selected_idx, file.file_name.clone()).into()
}

fn name_column_view(
    state: &AppState,
    side: Side,
    selected_idx: Option<usize>,
) -> Column<'_, Message> {
    let col = Column::new().push(header(state, "Name", 0));

    let names: Vec<Element<'_, Message>> = state
        .get_window_files(side)
        .iter()
        .enumerate()
        .map(|(idx, file)| name_view(side, idx, selected_idx, file))
        .collect();

    col.extend(names)
}

fn size_view(
    side: Side,
    idx: usize,
    selected_idx: Option<usize>,
    file: &FileRecord,
) -> Element<'_, Message> {
    let data = if file.is_go_back_record {
        "".to_string()
    } else {
        let size = file.size;
        let is_file = file.is_file;

        if is_file {
            format_size(size, DECIMAL.decimal_places(1))
        } else {
            "--".to_string()
        }
    };

    table_text_item(side, idx, selected_idx, data).into()
}

fn size_column_view(
    state: &AppState,
    side: Side,
    selected_idx: Option<usize>,
) -> Column<'_, Message> {
    let col = Column::new().push(header(state, "Size", 1));

    let names: Vec<Element<'_, Message>> = state
        .get_window_files(side)
        .iter()
        .enumerate()
        .map(|(idx, file)| size_view(side, idx, selected_idx, file))
        .collect();

    col.extend(names)
}

fn modified_view(
    side: Side,
    idx: usize,
    selected_idx: Option<usize>,
    file: &FileRecord,
) -> Element<'_, Message> {
    let data = if file.is_go_back_record {
        "".to_string()
    } else {
        let modified = file.modified;
        let datetime: DateTime<Local> = modified.into();

        datetime.format("%d %b %Y at %H:%M").to_string()
    };

    table_text_item(side, idx, selected_idx, data).into()
}

fn modified_column_view(
    state: &AppState,
    side: Side,
    selected_idx: Option<usize>,
) -> Column<'_, Message> {
    let col = Column::new().push(header(state, "Modified", 2));

    let names: Vec<Element<'_, Message>> = state
        .get_window_files(side)
        .iter()
        .enumerate()
        .map(|(idx, file)| modified_view(side, idx, selected_idx, file))
        .collect();

    col.extend(names)
}

pub fn table_view(state: &AppState, side: Side) -> Element<'_, Message> {
    let selected_idx = state.get_selected_idx(side);

    let name_col = name_column_view(state, side, selected_idx);
    let size_col = size_column_view(state, side, selected_idx);
    let modified_col = modified_column_view(state, side, selected_idx);

    let table = row![name_col, size_col, modified_col]; //.width(Fill);
    scrollable(container(table).padding(Padding::from([0, 10]))).into()
}
