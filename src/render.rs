use std::{cell::RefCell, path::PathBuf, rc::Rc};

use crate::{AppState, Side};

use imgui::*;

struct RenderTableResult {
    table_clicked: bool,
    to_open_idx: Option<usize>,
}

/// Renders files window (left or right).
pub fn render_files_window(
    ui: &Ui,
    state: Rc<RefCell<AppState>>,
    width: f32,
    height: f32,
    side: Side,
) -> Option<PathBuf> {
    let window_name: String = format!("{} window", side.as_str());
    let is_window_focused = state.borrow().is_window_focused(side);

    let mut path_to_open_option: Option<PathBuf> = None;

    ui.child_window(window_name)
        .size([width, height])
        .border(true)
        .focused(is_window_focused)
        .build(|| {
            {
                let mut state = state.borrow_mut();
                let files_len = state.get_window_files(side).len();

                let has_window_focus = ui.is_window_focused_with_flags(
                    imgui::WindowFocusedFlags::CHILD_WINDOWS,
                );

                ui.text(format!("Has focus: {has_window_focus}"));

                let current_item = state.get_selected_idx(side);

                if has_window_focus {
                    if ui.is_key_pressed(imgui::Key::DownArrow) {
                        let next_item = current_item + 1;
                        if next_item < files_len as i32 {
                            state.set_selected_idx(side, next_item);
                        }
                    } else if ui.is_key_pressed(imgui::Key::UpArrow) {
                        let prev_item = current_item - 1;
                        if prev_item >= 0 {
                            state.set_selected_idx(side, prev_item);
                        }
                    } else if ui.is_key_pressed(imgui::Key::Enter) {
                        // TODO: refactor, with below code when tab element is clicked
                        log::info!("{} table enter pressed", side.as_str());

                        let files = state.get_window_files(side);
                        let path = state.get_path(side);
                        let element_to_open = &files[current_item as usize];
                        let path_to_open: PathBuf =
                            [path, element_to_open].iter().collect();

                        path_to_open_option = Some(path_to_open);
                        // TODO: should i return early?
                    }
                }

                let path = state.get_path(side);
                ui.text(format!("Path: {path}"));
            }

            let render_table_result = render_table(ui, state.clone(), side);

            {
                let mut state = state.borrow_mut();

                if render_table_result.table_clicked {
                    log::debug!("{} table clicked", side.as_str());
                    state.focused_window_left = side.is_left();
                }

                if let Some(idx) = render_table_result.to_open_idx {
                    let files = state.get_window_files(side);
                    let path = state.get_path(side);
                    let element_to_open = &files[idx];
                    let path_to_open: PathBuf =
                        [path, element_to_open].iter().collect();

                    path_to_open_option = Some(path_to_open);
                }
            }
        });

    if ui.is_item_clicked() {
        log::debug!("{} window clicked", side.as_str());

        let mut state = state.borrow_mut();
        state.focused_window_left = side.is_left();
    }

    path_to_open_option
}

/// Renders table and some debug info about it.
fn render_table(
    ui: &Ui,
    state: Rc<RefCell<AppState>>,
    side: Side,
) -> RenderTableResult {
    let table_token = ui
        .begin_table_with_flags(
            "table",
            2,
            TableFlags::SORTABLE | TableFlags::RESIZABLE | TableFlags::ROW_BG,
        )
        .unwrap();

    let mut state = state.borrow_mut();
    let files = state.get_window_files(side);
    let mut current_item = state.get_selected_idx(side);

    let mut double_clicked_idx: Option<usize> = None;
    let mut any_row_clicked = false;

    for (idx, file) in files.iter().enumerate() {
        ui.table_next_row();
        ui.table_next_column();

        let clicked = ui
            .selectable_config(format!("{idx}"))
            .selected(idx == current_item as usize)
            .flags(
                SelectableFlags::SPAN_ALL_COLUMNS
                    | SelectableFlags::ALLOW_DOUBLE_CLICK,
            )
            .build();

        if clicked {
            log::info!("clicked idx: {idx}");
            if ui.is_mouse_double_clicked(MouseButton::Left) {
                double_clicked_idx = Some(idx);
            }

            current_item = idx as i32;
            any_row_clicked = true;
        }

        ui.table_next_column();
        ui.text(file);
    }

    table_token.end();

    state.set_selected_idx(side, current_item);

    ui.text(format!("current_item: {current_item}"));

    RenderTableResult {
        table_clicked: any_row_clicked,
        to_open_idx: double_clicked_idx,
    }
}
