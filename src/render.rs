use std::path::PathBuf;

use crate::ImguiState;

use imgui::*;

struct RenderTableResult {
    table_clicked: bool,
    to_open_idx: Option<usize>,
}

/// Renders files window (left or right).
pub unsafe fn render_files_window(
    ui_ptr: *mut Ui,
    imgui_ptr: *mut ImguiState,
    width: f32,
    height: f32,
    is_left: bool,
    path: &str,
    files: &Vec<String>,
) -> Option<PathBuf> {
    let ui: &mut Ui;
    let imgui: &mut ImguiState;

    unsafe {
        ui = ui_ptr.as_mut().unwrap();
        imgui = imgui_ptr.as_mut().unwrap();
    }

    let side = match is_left {
        true => "left",
        false => "right",
    };
    let window_name = format!("{side} window");
    let focused = match is_left {
        true => imgui.focused_window_left,
        false => !imgui.focused_window_left,
    };

    let mut path_to_open_option: Option<PathBuf> = None;

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

            let render_table_result;
            let current_item = match is_left {
                true => &mut imgui.left_item_selected_idx,
                false => &mut imgui.right_item_selected_idx,
            };

            ui.text(format!("Path: {path}"));

            unsafe {
                render_table_result = render_table(ui_ptr, files, current_item);
            }

            if render_table_result.table_clicked {
                log::debug!("{side} table clicked");
                imgui.focused_window_left = is_left
            }

            if let Some(idx) = render_table_result.to_open_idx {
                let element_to_open = &files[idx];
                let path_to_open: PathBuf = [path, element_to_open].iter().collect();

                path_to_open_option = Some(path_to_open);
            }
        });

    if ui.is_item_clicked() {
        log::debug!("{side} window clicked");
        imgui.focused_window_left = is_left;
    }

    path_to_open_option
}

/// Renders table and some debug info about it.
unsafe fn render_table(
    ui_ptr: *mut Ui,
    files: &Vec<String>,
    current_item: &mut i32,
) -> RenderTableResult {
    let ui: &mut Ui;

    unsafe {
        ui = ui_ptr.as_mut().unwrap();
    }

    let table_token = ui
        .begin_table_with_flags(
            "table",
            2,
            TableFlags::SORTABLE | TableFlags::RESIZABLE | TableFlags::ROW_BG,
        )
        .unwrap();

    let mut double_clicked_idx: Option<usize> = None;
    let mut any_row_clicked = false;
    for (idx, file) in files.iter().enumerate() {
        ui.table_next_row();
        ui.table_next_column();
        let clicked = ui
            .selectable_config(format!("{idx}"))
            .selected(idx == (*current_item) as usize)
            .flags(SelectableFlags::SPAN_ALL_COLUMNS | SelectableFlags::ALLOW_DOUBLE_CLICK)
            .build();

        if clicked {
            log::info!("clicked idx: {idx}");
            if ui.is_mouse_double_clicked(MouseButton::Left) {
                double_clicked_idx = Some(idx);
            }

            *current_item = idx as i32;
            any_row_clicked = true;
        }

        ui.table_next_column();
        ui.text(file);
    }

    table_token.end();

    ui.text(format!("current_item: {current_item}"));

    RenderTableResult {
        table_clicked: any_row_clicked,
        to_open_idx: double_clicked_idx,
    }
}
