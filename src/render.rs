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

        render_left_impl(ui, imgui_ptr, half_screen, main_window_h, files);
    }
}

fn render_left_impl(
    ui: &mut Ui,
    imgui_ptr: *mut ImguiState,
    half_screen: f32,
    main_window_h: f32,
    files: &Vec<String>,
) {
    let ui_ptr = ui as *mut Ui;

    ui.child_window("left window")
        .size([half_screen, main_window_h])
        .border(true)
        .build(|| unsafe {
            let imgui = imgui_ptr.as_mut().unwrap();
            let current_item = &mut imgui.left_item_selected_idx;

            let frame_rate = imgui.frame_rate;
            let frame_count = imgui.context.frame_count();

            ui.text(format!("Frame rate: {frame_rate} FPS"));
            ui.text(format!("Frame count: {frame_count}"));

            // render_frames_info(ui, frame_rate, frame_count);

            render_listbox(ui_ptr, files, current_item);
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
        render_right_impl(ui, imgui_ptr, main_window_h, files);
    }
}

fn render_right_impl(
    ui: &mut Ui,
    imgui_ptr: *mut ImguiState,
    main_window_h: f32,
    files: &Vec<String>,
) {
    let ui_ptr = ui as *mut Ui;

    ui.child_window("Right")
        .size([0., main_window_h])
        .border(true)
        .build(|| unsafe {
            let imgui = imgui_ptr.as_mut().unwrap();
            let current_item = &mut imgui.right_item_selected_idx;
            render_listbox(ui_ptr, files, current_item);
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

    log::trace!("{label} clicked: {clicked}");

    // ui.text(format!("clicked: {clicked}"));
    ui.text(format!("current_item: {current_item}"));
}
