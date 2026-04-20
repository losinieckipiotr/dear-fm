use chrono::{DateTime, Local};
use humansize::{DECIMAL, format_size};
use iced::Length::{self, Fill};
use iced::widget::text::Fragment;
use iced::widget::{
    Column, Row, button, container, mouse_area, row, scrollable, space, text,
};
use iced::{Background, Element, Padding, Theme, alignment};

use crate::Message;
use crate::files::{FileColumn, FileRecord, SortDirection, SortingOptions};
use crate::state::{AppState, Side};

fn sort_button(
    direction: &SortDirection,
    on_press: Message,
) -> Element<'_, Message> {
    let sort_icon = match direction {
        SortDirection::Ascending => "▲",
        SortDirection::Descending => "▼",
    };

    button(
        text(sort_icon)
            .width(Length::Fixed(22.0))
            .center()
            .wrapping(text::Wrapping::None),
    )
    .clip(true)
    .width(Length::Fixed(22.0))
    .padding(Padding::ZERO)
    .on_press(on_press)
    .into()
}

fn not_sorted_button(on_press: Message) -> Element<'static, Message> {
    button(
        text(" ")
            .width(Length::Fixed(20.0))
            .center()
            .wrapping(text::Wrapping::None),
    )
    .clip(true)
    .width(Length::Fixed(22.0))
    .padding(Padding::ZERO)
    .on_press(on_press)
    .into()
}

fn header<'a>(
    state: &'a AppState,
    side: Side,
    title: &'a str,
    file_col: FileColumn,
) -> Element<'a, Message> {
    let SortingOptions { sort_by, direction } = state.get_sorting_options(side);

    let button_element: Element<'a, Message> = match file_col {
        FileColumn::Name => {
            if let FileColumn::Name = sort_by {
                sort_button(
                    direction,
                    Message::SortRecords(
                        side,
                        SortingOptions {
                            sort_by: FileColumn::Name,
                            direction: direction.toggle(),
                        },
                    ),
                )
            } else {
                not_sorted_button(Message::SortRecords(
                    side,
                    SortingOptions {
                        sort_by: FileColumn::Name,
                        direction: SortDirection::Ascending,
                    },
                ))
            }
        }
        FileColumn::Size => {
            if let FileColumn::Size = sort_by {
                sort_button(
                    direction,
                    Message::SortRecords(
                        side,
                        SortingOptions {
                            sort_by: FileColumn::Size,
                            direction: direction.toggle(),
                        },
                    ),
                )
            } else {
                not_sorted_button(Message::SortRecords(
                    side,
                    SortingOptions {
                        sort_by: FileColumn::Size,
                        direction: SortDirection::Ascending,
                    },
                ))
            }
        }
        FileColumn::Modified => {
            if let FileColumn::Modified = sort_by {
                sort_button(
                    direction,
                    Message::SortRecords(
                        side,
                        SortingOptions {
                            sort_by: FileColumn::Modified,
                            direction: direction.toggle(),
                        },
                    ),
                )
            } else {
                not_sorted_button(Message::SortRecords(
                    side,
                    SortingOptions {
                        sort_by: FileColumn::Modified,
                        direction: SortDirection::Ascending,
                    },
                ))
            }
        }
    };

    container(
        Row::new()
            .push(text(title))
            .push(space::horizontal().width(Fill))
            .push(button_element)
            .align_y(alignment::Vertical::Center),
    )
    .padding(Padding::from([5, 5]))
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
    text_item: Fragment<'a>,
) -> Element<'a, Message> {
    let is_selected = state.is_selected_idx(side, idx);

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
                        Some(Background::Color(pallete.background.strong.color))
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
    .on_double_click(Message::RecordDoubleClick)
    .on_enter(Message::RecordHover(side, idx, file_col, true))
    .on_exit(Message::RecordHover(side, idx, file_col, false))
    .into()
}

fn name_view<'a>(
    state: &'a AppState,
    side: Side,
    idx: usize,
    file: &'a FileRecord,
) -> Element<'a, Message> {
    table_text_item(
        state,
        side,
        idx,
        FileColumn::Name,
        Fragment::Borrowed(&file.file_name),
    )
    .into()
}

fn name_column_view(state: &AppState, side: Side) -> Column<'_, Message> {
    let col = Column::new().push(header(state, side, "Name", FileColumn::Name));

    let names: Vec<Element<'_, Message>> = state
        .get_records(side)
        .iter()
        .enumerate()
        .map(|(idx, file)| name_view(state, side, idx, file))
        .collect();

    col.extend(names)
}

fn size_view<'a>(
    state: &'a AppState,
    side: Side,
    idx: usize,
    file: &'a FileRecord,
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
        Fragment::Owned(text_item),
    )
    .into()
}

fn size_column_view(state: &AppState, side: Side) -> Column<'_, Message> {
    let col = Column::new().push(header(state, side, "Size", FileColumn::Size));

    let names: Vec<Element<'_, Message>> = state
        .get_records(side)
        .iter()
        .enumerate()
        .map(|(idx, file)| size_view(state, side, idx, file))
        .collect();

    col.extend(names)
}

fn modified_view<'a>(
    state: &'a AppState,
    side: Side,
    idx: usize,
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
        Fragment::Owned(text_item),
    )
    .into()
}

fn modified_column_view<'a>(
    state: &'a AppState,
    side: Side,
) -> Column<'a, Message> {
    let col = Column::new().push(header(
        state,
        side,
        "Modified",
        FileColumn::Modified,
    ));

    let names: Vec<Element<'_, Message>> = state
        .get_records(side)
        .iter()
        .enumerate()
        .map(|(idx, file)| modified_view(state, side, idx, file))
        .collect();

    col.extend(names)
}

pub fn table_view(state: &AppState, side: Side) -> Element<'_, Message> {
    let name_col = name_column_view(state, side);
    let size_col = size_column_view(state, side);
    let modified_col = modified_column_view(state, side);

    let table = row![name_col, size_col, modified_col];
    scrollable(container(table).padding(Padding::from([0, 10]))).into()
}
