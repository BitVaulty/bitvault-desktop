mod app;
mod config;
mod icons;
mod wallet;

use eframe::egui;
use simple_logger::SimpleLogger;

fn main() {
    // Initialize logger with WARN level to reduce logging output
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    // Load settings for the initial window size
    let settings = config::Settings::load();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([settings.window_width, settings.window_height])
            .with_min_inner_size([400.0, 300.0]),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "BitVault",
        native_options,
        Box::new(|cc| Box::new(app::BitVaultApp::new(cc))),
    )
    .expect("Failed to start application");
}
