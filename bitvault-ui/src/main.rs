mod app;
mod crypto;
mod icons;
mod wallet;

use eframe::egui;
use simple_logger::SimpleLogger;

fn main() {
    // Initialize logger with WARN level to reduce logging output
    SimpleLogger::new()
        .with_level(log::LevelFilter::Warn)
        .init()
        .unwrap();

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
        Box::new(|cc| Box::new(app::BitVaultApp::new(cc))),
    )
    .expect("Failed to start application");
}
