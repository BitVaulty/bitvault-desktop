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
    if let Err(e) = SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init() {
        eprintln!("Failed to initialize logger: {}", e);
        // Continue without logger - not critical for app startup
    }

    // Create tokio runtime for async operations
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("Failed to create tokio runtime: {}", e);
            eprintln!("This is a critical error - the application cannot function without async support.");
            std::process::exit(1);
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0]),
        centered: true,
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "BitVault",
        native_options,
        Box::new(move |cc| {
            let mut app = app::BitVaultApp::new(cc);
            app.set_runtime(rt);
            Box::new(app)
        }),
    ) {
        eprintln!("Failed to start application: {}", e);
        std::process::exit(1);
    }
}

