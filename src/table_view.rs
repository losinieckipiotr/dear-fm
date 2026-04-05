use iced::Length::{Fill, Shrink};
use iced::widget::{container, row, rule, space, table, text};
use iced::{Element, Padding};

use crate::Message;
use crate::files::FileRecord;
use crate::state::{AppState, Side};

pub fn table_view(state: &AppState, side: Side) -> Element<'_, Message> {
    let columns = [
        table::column("file name", |file: &FileRecord| {
            text(file.file_name.clone())
        })
        .width(Shrink),
        table::column("size", |file: &FileRecord| {
            if file.is_go_back_record {
                text("")
            } else if file.is_file {
                // TODO: formatting
                text(file.size.to_string())
            } else {
                text("--")
            }
        })
        .width(Fill),
    ];

    let table = container(table(columns, state.get_window_files(side)))
        .padding(Padding::from([0, 10]));

    table.into()
}
