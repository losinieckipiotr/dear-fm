use std::{cell::RefCell, path::PathBuf, rc::Rc};

use crate::TestState;

use imgui::*;

struct RenderTableResult {
    table_clicked: bool,
    to_open_idx: Option<usize>,
    current_item: i32,
}

enum Side {
    Left,
    Right,
}

impl Side {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    fn from_bool(is_left: bool) -> Side {
        match is_left {
            true => Self::Left,
            false => Self::Right,
        }
    }
}

/// Renders files window (left or right).
pub fn render_files_window(
    ui: &Ui,
    test_state: Rc<RefCell<TestState>>,
    width: f32,
    height: f32,
    is_left: bool,
    // path: &str,
    // files: &Vec<String>,
) -> Option<PathBuf> {
    let side = Side::from_bool(is_left);
    let window_name: String = format!("{} window", side.as_str());
    let focused: bool;
    let path: String;
    let files: Vec<String>;
    // TODO: optimize
    {
        let state = test_state.borrow();

        (focused, path, files) = match side {
            Side::Left => (
                state.focused_window_left,
                state.app_files.left_path.clone(),
                state.app_files.left_files.clone(),
            ),
            Side::Right => (
                !state.focused_window_left,
                state.app_files.right_path.clone(),
                state.app_files.right_files.clone(),
            ),
        };
    }

    let mut path_to_open_option: Option<PathBuf> = None;

    ui.child_window(window_name)
        .size([width, height])
        .border(true)
        .focused(focused)
        .build(|| {
            let render_table_result: RenderTableResult;
            let current_item: i32;
            // TODO: move keyboard handling also to rendering table?
            {
                let mut state = test_state.borrow_mut();

                let has_focus = ui.is_window_focused_with_flags(
                    imgui::WindowFocusedFlags::CHILD_WINDOWS,
                );

                ui.text(format!("Has focus: {has_focus}"));

                let selected_item_ref = match is_left {
                    true => &mut state.left_item_selected_idx,
                    false => &mut state.right_item_selected_idx,
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

                current_item = *selected_item_ref;
            }

            ui.text(format!("Path: {path}"));

            render_table_result =
                render_table(ui, test_state.clone(), &files, current_item);

            {
                let mut state = test_state.borrow_mut();

                let selected_item_ref = match is_left {
                    true => &mut state.left_item_selected_idx,
                    false => &mut state.right_item_selected_idx,
                };

                *selected_item_ref = render_table_result.current_item;

                if render_table_result.table_clicked {
                    log::debug!("{} table clicked", side.as_str());
                    state.focused_window_left = is_left
                }

                if let Some(idx) = render_table_result.to_open_idx {
                    let element_to_open = &files[idx];
                    let path_to_open: PathBuf =
                        [&path, element_to_open].iter().collect();

                    path_to_open_option = Some(path_to_open);
                }
            }
        });

    if ui.is_item_clicked() {
        log::debug!("{} window clicked", side.as_str());

        let mut state = test_state.borrow_mut();

        state.focused_window_left = is_left;
    }

    path_to_open_option
}

/// Renders table and some debug info about it.
fn render_table(
    ui: &Ui,
    _test_state: Rc<RefCell<TestState>>,
    files: &Vec<String>,
    current_item: i32,
) -> RenderTableResult {
    let mut current_item = current_item;

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

    ui.text(format!("current_item: {current_item}"));

    // let test_state = test_state.borrow();
    // let value = test_state.demo_open;
    // let flag = test_state.frame_rate;

    // ui.text(format!("demo_open: {value}"));
    // ui.text(format!("frame_rate: {flag}"));

    RenderTableResult {
        table_clicked: any_row_clicked,
        to_open_idx: double_clicked_idx,
        current_item,
    }
}
