use chrono::{DateTime, Local};
use humansize::{DECIMAL, format_size};
use imgui::{
    MouseButton, SelectableFlags, TableColumnSetup, TableFlags,
    TableSortDirection, Ui,
};

use crate::{
    files::{FileRecord, SortBy, SortDirection},
    state::{AppState, Side},
};

pub fn render_table(ui: &Ui, state: &mut AppState, side: Side) {
    log::debug!("render_table");

    let table_token = ui
        .begin_table_header_with_flags(
            "table",
            [
                TableColumnSetup::new("Name"),
                TableColumnSetup::new("Size"),
                TableColumnSetup::new("Modified"),
            ],
            TableFlags::SORTABLE
                | TableFlags::RESIZABLE
                | TableFlags::ROW_BG
                | TableFlags::SIZING_FIXED_FIT,
        )
        .unwrap();

    handle_table_sorting(ui, state, side);

    let files = state.get_window_files(side);
    let mut selected_idx_option = state.get_selected_idx(side);
    let mut double_clicked_idx_option: Option<usize> = None;
    let mut any_row_clicked = false;

    for (idx, file) in files.iter().enumerate() {
        ui.table_next_row();

        render_name_column(
            ui,
            idx,
            file,
            &mut selected_idx_option,
            &mut double_clicked_idx_option,
            &mut any_row_clicked,
        );

        render_size_column(ui, file);
        render_modified_column(ui, file);
    }

    table_token.end();

    handle_table_interactions(
        state,
        side,
        selected_idx_option,
        any_row_clicked,
        double_clicked_idx_option,
    )
}

fn handle_table_sorting(ui: &Ui, state: &mut AppState, side: Side) {
    if let Some(sort_data) = ui.table_sort_specs_mut() {
        sort_data.conditional_sort(|specs| {
            let spec = specs.iter().next().unwrap();

            let get_sort_by = |column_idx: usize| -> SortBy {
                match column_idx {
                    0 => SortBy::Name,
                    1 => SortBy::Size,
                    2 => SortBy::Modified,
                    _ => unimplemented!(),
                }
            };

            if let Some(kind) = spec.sort_direction() {
                let (sort_by, direction) = match kind {
                    TableSortDirection::Ascending => (
                        get_sort_by(spec.column_idx()),
                        SortDirection::Ascending,
                    ),
                    TableSortDirection::Descending => (
                        get_sort_by(spec.column_idx()),
                        SortDirection::Descending,
                    ),
                };

                state.sort_window_files(side, sort_by, direction);
            }
        });
    }
}

fn render_name_column(
    ui: &Ui,
    idx: usize,
    file: &FileRecord,
    selected_idx_option: &mut Option<usize>,
    double_clicked_idx_option: &mut Option<usize>,
    any_row_clicked: &mut bool,
) {
    ui.table_next_column();

    let is_selected = match selected_idx_option {
        Some(selected_idx) => idx == *selected_idx,
        None => false,
    };

    let clicked = ui
        .selectable_config(&file.file_name)
        .selected(is_selected)
        .flags(
            SelectableFlags::SPAN_ALL_COLUMNS
                | SelectableFlags::ALLOW_DOUBLE_CLICK,
        )
        .build();

    if clicked {
        log::debug!("clicked idx: {idx}");

        if ui.is_mouse_double_clicked(MouseButton::Left) {
            *double_clicked_idx_option = Some(idx);
        }

        *selected_idx_option = Some(idx);
        *any_row_clicked = true;
    }
}

fn render_size_column(ui: &Ui, file: &FileRecord) {
    ui.table_next_column();

    if file.is_go_back_record {
        ui.text("");
    } else {
        let size = file.size;
        let is_file = file.is_file;

        let formatted_size: String =
            format_size(size, DECIMAL.decimal_places(1));
        if is_file {
            ui.text(formatted_size);
        } else {
            ui.text("--");
        }
    }
}

fn render_modified_column(ui: &Ui, file: &FileRecord) {
    ui.table_next_column();

    if file.is_go_back_record {
        ui.text("");
    } else {
        let modified = file.modified;
        let datetime: DateTime<Local> = modified.into();

        ui.text(format!("{}", datetime.format("%d %b %Y at %H:%M")));
    }
}

fn handle_table_interactions(
    state: &mut AppState,
    side: Side,
    selected_idx_option: Option<usize>,
    any_row_clicked: bool,
    double_clicked_idx_option: Option<usize>,
) {
    if let Some(selected_idx) = selected_idx_option {
        state.set_selected_idx(side, selected_idx);
    }

    if any_row_clicked {
        log::debug!("{} table clicked", side);

        state.focus_window(side);
    }

    if let Some(idx) = double_clicked_idx_option {
        let path_to_open = state.get_path_to_open_at(side, idx);

        state.go_to_or_open(side, path_to_open);
    }
}
