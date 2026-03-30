use crate::{
    render::render_panel,
    state::{AppState, Side},
};
use std::path::PathBuf;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    state: AppState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: AppState::default(),
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app: Self = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        app.state.go_to_directory(
            Side::Left,
            PathBuf::from("/Users/piotrlosiniecki"),
        );
        app.state.go_to_directory(
            Side::Right,
            PathBuf::from("/Users/piotrlosiniecki/Projects"),
        );

        app
    }
}

impl eframe::App for App {
    fn persist_egui_memory(&self) -> bool {
        true
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        log::info!("quit button clicked");
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        render_panel(ui, Side::Left, &mut self.state);
        render_panel(ui, Side::Right, &mut self.state);

        // TODO: bottom panel
        // ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        //     powered_by_egui_and_eframe(ui);
        //     egui::warn_if_debug_build(ui);
        // });
    }
}

// fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
//     ui.horizontal(|ui| {
//         ui.spacing_mut().item_spacing.x = 0.0;
//         ui.label("Powered by ");
//         ui.hyperlink_to("egui", "https://github.com/emilk/egui");
//         ui.label(" and ");
//         ui.hyperlink_to(
//             "eframe",
//             "https://github.com/emilk/egui/tree/master/crates/eframe",
//         );
//         ui.label(".");
//     });
// }
