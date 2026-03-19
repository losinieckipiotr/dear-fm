use crate::ImguiState;
use imgui::*;

pub unsafe fn render_right(ui: *mut Ui, main_window_h: f32) {
    unsafe {
        let ui = ui.as_mut().unwrap();
        render_right_impl(ui, main_window_h);
    }
}

fn render_right_impl(ui: &mut Ui, main_window_h: f32) {
    ui.child_window("Right")
        .size([0., main_window_h])
        .border(true)
        .build(|| {
            for i in 0..100 {
                ui.text(format!("{i}_right_file.xdd"));
            }
        });
}

pub unsafe fn render_listbox(ui: *mut Ui, imgui: *mut ImguiState) {
    unsafe {
        let ui = ui.as_mut().unwrap();
        let imgui = imgui.as_mut().unwrap();
        render_listbox_impl(ui, imgui);
    }
}

fn render_listbox_impl(ui: &mut Ui, imgui: &mut ImguiState) {
    const LEFT_ITEMS_NUM: usize = 200;
    let current_item = &mut imgui.left_item_selected_idx;
    let mut items = Vec::with_capacity(LEFT_ITEMS_NUM);
    for i in 0..LEFT_ITEMS_NUM {
        let item = format!("{i}_lef_file.xdd");
        items.push(item);
    }
    let items_strs: Vec<&str> = items.iter().map(|item| item.as_str()).collect();

    imgui::Ui::set_next_item_width(ui, -1.0);
    let clicked = ui.list_box(
        "##left listbox",
        current_item,
        items_strs.as_slice(),
        LEFT_ITEMS_NUM as i32,
    );

    ui.text(format!("clicked: {clicked}"));
    ui.text(format!("current_item: {current_item}"));
}
