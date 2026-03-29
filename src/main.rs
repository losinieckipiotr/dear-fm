#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// mod files;
// mod render_files_window;
// mod render_frame;
// mod render_table;
// mod state;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    // env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    env_logger::init_from_env(
        env_logger::Env::new().default_filter_or(log::Level::Info.as_str()),
    );

    let version = env!("CARGO_PKG_VERSION");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(
                    &include_bytes!("../assets/favicon-512x512.png")[..],
                )
                .expect("Failed to load icon"),
            ),
        // TODO check persistence_path
        ..Default::default()
    };

    eframe::run_native(
        format!("Dear File Manager {version}").as_str(),
        native_options,
        Box::new(|cc| Ok(Box::new(dear_fm::App::new(cc)))),
    )
}
