#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

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
                eframe::icon_data::from_png_bytes(
                    &include_bytes!("../assets/favicon-512x512.png")[..],
                )
                .expect("Failed to load icon"),
            ),
        persistence_path: Some(PathBuf::from("dear-fm.ron")),
        ..Default::default()
    };

    eframe::run_native(
        format!("Dear File Manager {version}").as_str(),
        native_options,
        Box::new(|cc| Ok(Box::new(dear_fm::App::new(cc)))),
    )
}
