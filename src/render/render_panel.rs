use crate::render::render_table;
use crate::state::{AppState, Side};
use egui::{CentralPanel, Panel, ScrollArea, Ui};

pub fn render_panel(ui: &mut Ui, side: Side, state: &mut AppState) {
    let available_width = ui.available_width();

    match side {
        Side::Left => {
            Panel::left(format!("{}_panel", side))
                .resizable(true)
                .default_size(available_width / 2.0)
                .min_size(200.0)
                .max_size(available_width - 200.0)
                .show_inside(ui, |ui| {
                    render_inside(ui, side, state);
                });
        }
        Side::Right => {
            CentralPanel::default().show_inside(ui, |ui| {
                ui.take_available_width();

                render_inside(ui, side, state);
            });
        }
    }
}

fn render_inside(ui: &mut Ui, side: Side, state: &mut AppState) {
    ui.vertical_centered(|ui| ui.heading(format!("{} panel", side)));

    ScrollArea::vertical().show(ui, |ui| {
        render_table(ui, side, state);
    });
}
