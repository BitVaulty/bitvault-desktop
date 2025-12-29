mod app;
mod services;
mod models;
mod utils;
mod state;
mod ui;
mod settings;

use eframe::egui;
use simple_logger::SimpleLogger;
use log::LevelFilter;

fn main() {
    // Initialize logger
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    // Create tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new().unwrap();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0]),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "BitVault",
        native_options,
        Box::new(move |cc| {
            let mut app = app::BitVaultApp::new(cc);
            app.set_runtime(rt);
            Box::new(app)
        }),
    )
    .expect("Failed to start application");
}

