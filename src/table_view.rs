use chrono::{DateTime, Local};
use humansize::{DECIMAL, format_size};
use iced::Length::Fill;
use iced::widget::{
    Column, Row, button, container, mouse_area, opaque, row, scrollable, space,
    text,
};
use iced::{Background, Element, Padding, Theme, alignment};

use crate::Message;
use crate::files::{FileColumn, FileRecord, SortBy, SortDirection};
use crate::state::{AppState, Side, SortingOptions};

fn header<'a>(
    side: Side,
    title: &'a str,
    file_col: FileColumn,
    sort_by: SortBy,
    direction: SortDirection,
) -> Element<'a, Message> {
    let sort_icon = match direction {
        SortDirection::Ascending => "/\\",
        SortDirection::Descending => "\\/",
    };

    // TODO: refactor
    let sort_button: Element<'a, Message> = match file_col {
        FileColumn::Name => {
            if let SortBy::Name = sort_by {
                button(text(sort_icon))
                    .on_press(Message::SortRecords(
                        side,
                        SortBy::Name,
                        match direction {
                            SortDirection::Ascending => {
                                SortDirection::Descending
                            }
                            SortDirection::Descending => {
                                SortDirection::Ascending
                            }
                        },
                    ))
                    .into()
            } else {
                button(text("-"))
                    .on_press(Message::SortRecords(
                        side,
                        SortBy::Name,
                        SortDirection::Ascending,
                    ))
                    .into()
            }
        }
        FileColumn::Size => {
            if let SortBy::Size = sort_by {
                button(text(sort_icon))
                    .on_press(Message::SortRecords(
                        side,
                        SortBy::Size,
                        match direction {
                            SortDirection::Ascending => {
                                SortDirection::Descending
                            }
                            SortDirection::Descending => {
                                SortDirection::Ascending
                            }
                        },
                    ))
                    .into()
            } else {
                button(text("-"))
                    .on_press(Message::SortRecords(
                        side,
                        SortBy::Size,
                        SortDirection::Ascending,
                    ))
                    .into()
            }
        }
        FileColumn::Modified => {
            if let SortBy::Modified = sort_by {
                button(text(sort_icon))
                    .on_press(Message::SortRecords(
                        side,
                        SortBy::Modified,
                        match direction {
                            SortDirection::Ascending => {
                                SortDirection::Descending
                            }
                            SortDirection::Descending => {
                                SortDirection::Ascending
                            }
                        },
                    ))
                    .into()
            } else {
                button(text("-"))
                    .on_press(Message::SortRecords(
                        side,
                        SortBy::Modified,
                        SortDirection::Ascending,
                    ))
                    .into()
            }
        }
    };

    container(
        Row::new()
            .push(text(title))
            .push(space::horizontal().width(Fill))
            .push(sort_button)
            .align_y(alignment::Vertical::Center),
    )
    .padding(Padding::from([0, 5])) // TODO: container with padding only for title
    .align_left(Fill)
    .style(|theme: &Theme| container::Style {
        background: Some(Background::Color(
            theme.extended_palette().background.strong.color,
        )),
        ..Default::default()
    })
    .into()
}

fn table_text_item<'a>(
    state: &'a AppState,
    side: Side,
    idx: usize,
    file_col: FileColumn,
    selected_idx: Option<usize>,
    file_name: String,
    text_item: String,
) -> Element<'a, Message> {
    let is_selected = match selected_idx {
        Some(selected_idx) => selected_idx == idx,
        None => false,
    };

    opaque(
        mouse_area(
            container(text(text_item).wrapping(text::Wrapping::None))
                .width(Fill)
                .style(move |theme: &Theme| {
                    let pallete = theme.extended_palette();
                    let hover = state.get_hover_for_idx(side, idx);

                    let background: Option<Background> = if is_selected {
                        Some(Background::Color(pallete.primary.base.color))
                    } else {
                        if hover {
                            Some(Background::Color(
                                pallete.background.strong.color,
                            ))
                        } else {
                            None
                        }
                    };

                    container::Style {
                        background,
                        ..Default::default()
                    }
                })
                .clip(true),
        )
        .on_press(Message::SelectRecord(side, idx))
        .on_double_click(Message::RecordDoubleClick(side, file_name))
        .on_enter(Message::RecordHover(side, idx, file_col, true))
        .on_exit(Message::RecordHover(side, idx, file_col, false)),
    )
    .into()
}

fn name_view<'a>(
    state: &'a AppState,
    side: Side,
    idx: usize,
    selected_idx: Option<usize>,
    file: &'a FileRecord,
) -> Element<'a, Message> {
    table_text_item(
        state,
        side,
        idx,
        FileColumn::Name,
        selected_idx,
        file.file_name.clone(),
        file.file_name.clone(),
    )
    .into()
}

fn name_column_view(
    state: &AppState,
    side: Side,
    selected_idx: Option<usize>,
) -> Column<'_, Message> {
    let SortingOptions { sort_by, direction } = state.get_sorting_options(side);

    let col = Column::new().push(header(
        side,
        "Name",
        FileColumn::Name,
        sort_by,
        direction,
    ));

    let names: Vec<Element<'_, Message>> = state
        .get_records(side)
        .iter()
        .enumerate()
        .map(|(idx, file)| name_view(state, side, idx, selected_idx, file))
        .collect();

    col.extend(names)
}

fn size_view<'a>(
    state: &'a AppState,
    side: Side,
    idx: usize,
    selected_idx: Option<usize>,
    file: &FileRecord,
) -> Element<'a, Message> {
    let text_item = if file.is_go_back_record {
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

    table_text_item(
        state,
        side,
        idx,
        FileColumn::Size,
        selected_idx,
        file.file_name.clone(),
        text_item,
    )
    .into()
}

fn size_column_view(
    state: &AppState,
    side: Side,
    selected_idx: Option<usize>,
) -> Column<'_, Message> {
    let SortingOptions { sort_by, direction } = state.get_sorting_options(side);

    let col = Column::new().push(header(
        side,
        "Size",
        FileColumn::Size,
        sort_by,
        direction,
    ));

    let names: Vec<Element<'_, Message>> = state
        .get_records(side)
        .iter()
        .enumerate()
        .map(|(idx, file)| size_view(state, side, idx, selected_idx, file))
        .collect();

    col.extend(names)
}

fn modified_view<'a>(
    state: &'a AppState,
    side: Side,
    idx: usize,
    selected_idx: Option<usize>,
    file: &'a FileRecord,
) -> Element<'a, Message> {
    let text_item = if file.is_go_back_record {
        "".to_string()
    } else {
        let modified = file.modified;
        let datetime: DateTime<Local> = modified.into();

        datetime.format("%d %b %Y at %H:%M").to_string()
    };

    table_text_item(
        state,
        side,
        idx,
        FileColumn::Modified,
        selected_idx,
        file.file_name.clone(),
        text_item,
    )
    .into()
}

fn modified_column_view<'a>(
    state: &'a AppState,
    side: Side,
    selected_idx: Option<usize>,
) -> Column<'a, Message> {
    let SortingOptions { sort_by, direction } = state.get_sorting_options(side);

    let col = Column::new().push(header(
        side,
        "Modified",
        FileColumn::Modified,
        sort_by,
        direction,
    ));

    let names: Vec<Element<'_, Message>> = state
        .get_records(side)
        .iter()
        .enumerate()
        .map(|(idx, file)| modified_view(state, side, idx, selected_idx, file))
        .collect();

    col.extend(names)
}

pub fn table_view(state: &AppState, side: Side) -> Element<'_, Message> {
    let selected_idx = state.get_selected_idx(side);

    let name_col = name_column_view(state, side, selected_idx);
    let size_col = size_column_view(state, side, selected_idx);
    let modified_col = modified_column_view(state, side, selected_idx);

    let table = row![name_col, size_col, modified_col];
    scrollable(container(table).padding(Padding::from([0, 10]))).into()
}
