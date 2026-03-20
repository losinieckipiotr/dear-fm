use crate::ImguiState;

use imgui::*;

pub unsafe fn render_left(
    ui_ptr: *mut Ui,
    imgui_ptr: *mut ImguiState,
    half_screen: f32,
    main_window_h: f32,
    files: &Vec<String>,
) {
    unsafe {
        let ui = ui_ptr.as_mut().unwrap();
        let imgui = imgui_ptr.as_mut().unwrap();
        let focused = (*imgui_ptr).focused_window_left;

        render_left_impl(ui, imgui, half_screen, main_window_h, files, focused);
    }
}

fn render_left_impl(
    ui: &mut Ui,
    imgui: &mut ImguiState,
    half_screen: f32,
    main_window_h: f32,
    files: &Vec<String>,
    focused: bool,
) {
    let ui_ptr = ui as *mut Ui;

    ui.child_window("left window")
        .size([half_screen, main_window_h])
        .border(true)
        .focused(focused)
        .build(|| {
            let frame_rate = imgui.frame_rate;
            let frame_count = imgui.context.frame_count();
            let has_focus =
                ui.is_window_focused_with_flags(imgui::WindowFocusedFlags::CHILD_WINDOWS);

            ui.text(format!("Frame rate: {frame_rate} FPS"));
            ui.text(format!("Frame count: {frame_count}"));
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

            let current_item = &mut imgui.left_item_selected_idx;

            unsafe {
                render_listbox(ui_ptr, files, current_item);
            }
        });
}

pub unsafe fn render_right(
    ui: *mut Ui,
    imgui_ptr: *mut ImguiState,
    main_window_h: f32,
    files: &Vec<String>,
) {
    unsafe {
        let ui = ui.as_mut().unwrap();
        let imgui = imgui_ptr.as_mut().unwrap();
        let focused = !(*imgui_ptr).focused_window_left;

        render_right_impl(ui, imgui, main_window_h, files, focused);
    }
}

fn render_right_impl(
    ui: &mut Ui,
    imgui: &mut ImguiState,
    main_window_h: f32,
    files: &Vec<String>,
    focused: bool,
) {
    let ui_ptr = ui as *mut Ui;

    ui.child_window("Right")
        .size([0., main_window_h])
        .border(true)
        .focused(focused)
        .build(|| {
            let active = ui.is_item_active();
            ui.text(format!("Active: {active}"));

            let has_focus =
                ui.is_window_focused_with_flags(imgui::WindowFocusedFlags::CHILD_WINDOWS);
            let hovered = ui.is_window_hovered_with_flags(imgui::WindowHoveredFlags::CHILD_WINDOWS);
            let clicked = ui.is_mouse_clicked(imgui::MouseButton::Left);

            ui.text(format!("Has focus: {has_focus}"));
            ui.text(format!("Hovered: {hovered}"));
            ui.text(format!("Clicked: {clicked}"));

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

            let current_item = &mut imgui.right_item_selected_idx;

            unsafe {
                render_listbox(ui_ptr, files, current_item);
            }
        });
}

pub unsafe fn render_listbox(ui: *mut Ui, files: &Vec<String>, current_item: &mut i32) {
    unsafe {
        let ui = ui.as_mut().unwrap();
        render_listbox_impl(ui, files, current_item);
    }
}

fn render_listbox_impl(
    ui: &mut Ui,
    // imgui: &mut ImguiState,
    files: &Vec<String>,
    current_item: &mut i32,
) {
    // let current_item = &mut imgui.left_item_selected_idx;
    let items_strs: Vec<&str> = files.iter().map(|i| i.as_str()).collect();

    imgui::Ui::set_next_item_width(ui, -1.0);
    let label = "##left listbox";
    let clicked = ui.list_box(
        label,
        current_item,
        items_strs.as_slice(),
        items_strs.len() as i32,
    );

    let is_listbox_active = ui.is_item_active();
    ui.text(format!("Listbox active: {is_listbox_active}"));

    log::trace!("{label} clicked: {clicked}");

    // ui.text(format!("clicked: {clicked}"));
    ui.text(format!("current_item: {current_item}"));
}
