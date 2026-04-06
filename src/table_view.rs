use chrono::{DateTime, Local};
use humansize::{DECIMAL, format_size};
use iced::Length::{Fill, Shrink};
use iced::widget::{button, container, mouse_area, opaque, table, text};
use iced::{Element, Padding};

use crate::Message;
use crate::files::FileRecord;
use crate::state::{AppState, Side};

#[derive(Debug, Clone)]
struct Record {
    idx: usize,
    file: FileRecord,
}

pub fn table_view(state: &AppState, side: Side) -> Element<'_, Message> {
    let columns = [
        table::column("Name", |r: Record| name_view(side, r.idx, r.file))
            .width(Shrink),
        table::column("Size", |r: Record| size_view(r.file)),
        table::column("Modified", |r: Record| modified_view(r.file))
            .width(Fill),
    ];

    let data: Vec<Record> = state
        .get_window_files(side)
        .iter()
        .enumerate()
        .map(|(idx, file)| Record {
            idx,
            file: file.clone(),
        })
        .collect();

    container(table(columns, data))
        .padding(Padding::from([0, 10]))
        .into()
}

fn name_view(
    side: Side,
    idx: usize,
    file: FileRecord,
) -> Element<'static, Message> {
    button(text(file.file_name))
        .on_press(Message::OpenFileOrDir(side, idx))
        .into()
}

fn size_view(file: FileRecord) -> Element<'static, Message> {
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

fn modified_view(file: FileRecord) -> Element<'static, Message> {
    let view = if file.is_go_back_record {
        text("")
    } else {
        let modified = file.modified;
        let datetime: DateTime<Local> = modified.into();

        text(datetime.format("%d %b %Y at %H:%M").to_string())
    };

    opaque(mouse_area(view).on_press(Message::TestClick)).into()
    // view.into()
}
