use crate::ImguiState;

use imgui::*;

pub unsafe fn render_left(
    ui_ptr: *mut Ui,
    imgui_ptr: *mut ImguiState,
    half_screen: f32,
    main_window_h: f32,
    path: &str,
    files: &Vec<String>,
) {
    unsafe {
        let ui = ui_ptr.as_mut().unwrap();
        let imgui = imgui_ptr.as_mut().unwrap();
        let focused = (*imgui_ptr).focused_window_left;

        render_left_impl(ui, imgui, half_screen, main_window_h, path, files, focused);
    }
}

fn render_left_impl(
    ui: &mut Ui,
    imgui: &mut ImguiState,
    half_screen: f32,
    main_window_h: f32,
    path: &str,
    files: &Vec<String>,
    focused: bool,
) {
    let ui_ptr = ui as *mut Ui;

    ui.child_window("left")
        .size([half_screen, main_window_h])
        .border(true)
        .focused(focused)
        .build(|| {
            let has_focus =
                ui.is_window_focused_with_flags(imgui::WindowFocusedFlags::CHILD_WINDOWS);

            ui.text(format!("Has focus: {has_focus}"));

            if has_focus {
                if ui.is_key_pressed(imgui::Key::DownArrow) {
                    let next_item = imgui.left_item_selected_idx + 1;
                    if next_item < files.len() as i32 {
                        imgui.left_item_selected_idx = next_item
                    }
                }
                if ui.is_key_pressed(imgui::Key::UpArrow) {
                    let prev_item = imgui.left_item_selected_idx - 1;
                    if prev_item >= 0 {
                        imgui.left_item_selected_idx = prev_item
                    }
                }
            }

            let clicked;
            let current_item = &mut imgui.left_item_selected_idx;

            ui.text(format!("Path: {path}"));

            unsafe {
                clicked = render_table(ui_ptr, files, current_item);
            }

            if clicked {
                log::info!("left table clicked, focus left window");
                imgui.focused_window_left = true;
            }
        });

    if ui.is_item_clicked() {
        {
            log::debug!("left window clicked, focus left window");
            imgui.focused_window_left = true;
        }
    }
}

pub unsafe fn render_right(
    ui: *mut Ui,
    imgui_ptr: *mut ImguiState,
    main_window_h: f32,
    path: &str,
    files: &Vec<String>,
) {
    unsafe {
        let ui = ui.as_mut().unwrap();
        let imgui = imgui_ptr.as_mut().unwrap();
        let focused = !(*imgui_ptr).focused_window_left;

        render_right_impl(ui, imgui, main_window_h, path, files, focused);
    }
}

fn render_right_impl(
    ui: &mut Ui,
    imgui: &mut ImguiState,
    main_window_h: f32,
    path: &str,
    files: &Vec<String>,
    focused: bool,
) {
    let ui_ptr = ui as *mut Ui;

    ui.child_window("right")
        .size([0., main_window_h])
        .border(true)
        .focused(focused)
        .build(|| {
            let has_focus =
                ui.is_window_focused_with_flags(imgui::WindowFocusedFlags::CHILD_WINDOWS);

            ui.text(format!("Has focus: {has_focus}"));

            if has_focus {
                if ui.is_key_pressed(imgui::Key::DownArrow) {
                    let next_item = imgui.right_item_selected_idx + 1;
                    if next_item < files.len() as i32 {
                        imgui.right_item_selected_idx = next_item
                    }
                }

                if ui.is_key_pressed(imgui::Key::UpArrow) {
                    let prev_item = imgui.right_item_selected_idx - 1;
                    if prev_item >= 0 {
                        imgui.right_item_selected_idx = prev_item
                    }
                }
            }

            let clicked;
            let current_item = &mut imgui.right_item_selected_idx;

            ui.text(format!("Path: {path}"));

            unsafe {
                clicked = render_table(ui_ptr, files, current_item);
            }

            if clicked {
                log::debug!("right table clicked, focus right window");
                imgui.focused_window_left = false;
            }
        });

    if ui.is_item_clicked() {
        log::debug!("right window clicked, focus right window");
        imgui.focused_window_left = false;
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
            "table left",
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
