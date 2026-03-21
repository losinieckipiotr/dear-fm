use crate::ImguiState;

use imgui::*;

pub unsafe fn render_files_window(
    ui_ptr: *mut Ui,
    imgui_ptr: *mut ImguiState,
    width: f32,
    height: f32,
    is_left: bool,
    path: &str,
    files: &Vec<String>,
) {
    unsafe {
        let ui = ui_ptr.as_mut().unwrap();
        let imgui = imgui_ptr.as_mut().unwrap();

        render_window_impl(ui, imgui, width, height, is_left, path, files);
    }
}

fn render_window_impl(
    ui: &mut Ui,
    imgui: &mut ImguiState,
    width: f32,
    height: f32,
    is_left: bool,
    path: &str,
    files: &Vec<String>,
) {
    let ui_ptr: *mut Ui = ui as *mut Ui;

    let side = match is_left {
        true => "left",
        false => "right",
    };
    let window_name = format!("{side} window");
    let focused = match is_left {
        true => imgui.focused_window_left,
        false => !imgui.focused_window_left,
    };

    ui.child_window(window_name)
        .size([width, height])
        .border(true)
        .focused(focused)
        .build(|| {
            let has_focus =
                ui.is_window_focused_with_flags(imgui::WindowFocusedFlags::CHILD_WINDOWS);

            ui.text(format!("Has focus: {has_focus}"));

            let selected_item_ref = match is_left {
                true => &mut imgui.left_item_selected_idx,
                false => &mut imgui.right_item_selected_idx,
            };

            if has_focus {
                if ui.is_key_pressed(imgui::Key::DownArrow) {
                    let next_item = *selected_item_ref + 1;
                    if next_item < files.len() as i32 {
                        *selected_item_ref = next_item
                    }
                }
                if ui.is_key_pressed(imgui::Key::UpArrow) {
                    let prev_item = *selected_item_ref - 1;
                    if prev_item >= 0 {
                        *selected_item_ref = prev_item
                    }
                }
            }

            let clicked;
            let current_item = match is_left {
                true => &mut imgui.left_item_selected_idx,
                false => &mut imgui.right_item_selected_idx,
            };

            ui.text(format!("Path: {path}"));

            unsafe {
                clicked = render_table(ui_ptr, files, current_item);
            }

            if clicked {
                log::debug!("{side} table clicked");
                imgui.focused_window_left = is_left
            }
        });

    if ui.is_item_clicked() {
        log::debug!("{side} window clicked");
        imgui.focused_window_left = is_left;
    }
}

/// Renders table and some debug info about it. Returns true if table was clicked.
unsafe fn render_table(ui: *mut Ui, files: &Vec<String>, current_item: &mut i32) -> bool {
    unsafe {
        let ui = ui.as_mut().unwrap();
        return render_table_impl(ui, files, current_item);
    }
}

fn render_table_impl(ui: &mut Ui, files: &Vec<String>, current_item: &mut i32) -> bool {
    let table_token = ui
        .begin_table_with_flags(
            "table",
            2,
            TableFlags::SORTABLE | TableFlags::RESIZABLE | TableFlags::ROW_BG,
        )
        .unwrap();

    let mut any_row_clicked = false;
    for (idx, file) in files.iter().enumerate() {
        ui.table_next_row();
        ui.table_next_column();
        let clicked = ui
            .selectable_config(format!("{idx}"))
            .selected(idx == (*current_item) as usize)
            .flags(SelectableFlags::SPAN_ALL_COLUMNS)
            .build();

        if clicked {
            *current_item = idx as i32;
            any_row_clicked = true;
        }

        ui.table_next_column();
        ui.text(file);
    }

    table_token.end();

    ui.text(format!("current_item: {current_item}"));

    any_row_clicked
}
