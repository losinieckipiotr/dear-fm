use crate::ImguiState;

use imgui::*;

pub unsafe fn render_left(
    ui: *mut Ui,
    imgui_ptr: *mut ImguiState,
    half_screen: f32,
    main_window_h: f32,
    last_frame_rate: u32,
    frame_count: i32,
    files: &Vec<String>,
) {
    unsafe {
        let ui = ui.as_mut().unwrap();
        render_left_impl(
            ui,
            imgui_ptr,
            half_screen,
            main_window_h,
            last_frame_rate,
            frame_count,
            files,
        );
    }
}

fn render_left_impl(
    ui: &mut Ui,
    imgui_ptr: *mut ImguiState,
    half_screen: f32,
    main_window_h: f32,
    last_frame_rate: u32,
    frame_count: i32,
    files: &Vec<String>,
) {
    let ui_ptr = ui as *mut Ui;

    ui.child_window("left window")
        .size([half_screen, main_window_h])
        .border(true)
        .build(|| {
            ui.text(format!("Frame rate: {last_frame_rate} FPS"));
            ui.text(format!("Frame count: {frame_count}"));

            // if let Err(error) = files::read_directory() {
            //     log::error!("error during firectory read: {:#?}", error);
            //     return;
            // }

            unsafe {
                render_listbox(ui_ptr, imgui_ptr, files);
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
        .build(|| {
            // files.iter().for_each(|file| {
            //     ui.text(file);
            // });

            unsafe {
                render_listbox(ui_ptr, imgui_ptr, files);
            }
        });
}

pub unsafe fn render_listbox(ui: *mut Ui, imgui: *mut ImguiState, files: &Vec<String>) {
    unsafe {
        let ui = ui.as_mut().unwrap();
        let imgui = imgui.as_mut().unwrap();
        render_listbox_impl(ui, imgui, files);
    }
}

fn render_listbox_impl(ui: &mut Ui, imgui: &mut ImguiState, files: &Vec<String>) {
    // const LEFT_ITEMS_NUM: usize = 200;
    let current_item = &mut imgui.left_item_selected_idx;
    // let mut items = Vec::with_capacity(LEFT_ITEMS_NUM);
    // for i in 0..LEFT_ITEMS_NUM {
    //     let item = format!("{i}_lef_file.xdd");
    //     items.push(item);
    // }
    // let items_strs: Vec<&str> = items.iter().map(|item| item.as_str()).collect();
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
    // ui.text(format!("current_item: {current_item}"));
}
