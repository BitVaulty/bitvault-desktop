mod app;
mod models;
mod services;
mod settings;
mod state;
mod ui;
mod utils;

use eframe::egui;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::path::PathBuf;

/// Try to load the app icon (BV logo)
/// Looks for app.ico or app.png in the resources directory
fn load_app_icon() -> Option<egui::IconData> {
    // Try to find the icon file - prioritize smaller icons first (better for window managers)
    let mut possible_paths = vec![
        // Relative to workspace root - try smaller icons first
        PathBuf::from("bitvault-desktop/bitvault-app/resources/app_64.png"),
        PathBuf::from("bitvault-desktop/bitvault-app/resources/app_128.png"),
        PathBuf::from("bitvault-desktop/bitvault-app/resources/app.ico"),
        PathBuf::from("bitvault-desktop/bitvault-app/resources/app.png"),
        // Relative to current working directory
        PathBuf::from("resources/app_64.png"),
        PathBuf::from("resources/app_128.png"),
        PathBuf::from("resources/app.ico"),
        PathBuf::from("resources/app.png"),
        PathBuf::from("bitvault-app/resources/app_64.png"),
        PathBuf::from("bitvault-app/resources/app_128.png"),
        PathBuf::from("bitvault-app/resources/app.ico"),
        PathBuf::from("bitvault-app/resources/app.png"),
    ];

    // Also try in the executable directory
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            possible_paths.push(exe_dir.join("resources/app_64.png"));
            possible_paths.push(exe_dir.join("resources/app_128.png"));
            possible_paths.push(exe_dir.join("resources/app.ico"));
            possible_paths.push(exe_dir.join("resources/app.png"));

            // If we're in target/release, go up to find bitvault-app/resources
            let mut current = exe_dir;
            while let Some(parent) = current.parent() {
                let bitvault_app_resources = parent.join("bitvault-app/resources/app.ico");
                if bitvault_app_resources.exists() {
                    possible_paths.push(parent.join("bitvault-app/resources/app_64.png"));
                    possible_paths.push(parent.join("bitvault-app/resources/app_128.png"));
                    possible_paths.push(bitvault_app_resources.clone());
                    possible_paths.push(parent.join("bitvault-app/resources/app.png"));
                    break;
                }
                if parent == current || !parent.exists() {
                    break;
                }
                current = parent;
            }
        }
    }

    for path in possible_paths.iter() {
        if path.exists() {
            eprintln!("Trying to load icon from: {}", path.display());
            if let Some(icon_data) = crate::utils::images::load_icon_data(path) {
                eprintln!(
                    "Successfully loaded icon from: {} ({}x{})",
                    path.display(),
                    icon_data.width,
                    icon_data.height
                );
                return Some(icon_data);
            } else {
                eprintln!("Failed to load icon from: {}", path.display());
            }
        }
    }

    None
}

fn main() {
    // Initialize logger
    if let Err(e) = SimpleLogger::new().with_level(LevelFilter::Info).init() {
        eprintln!("Failed to initialize logger: {}", e);
        // Continue without logger - not critical for app startup
    }

    // Create tokio runtime for async operations
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("Failed to create tokio runtime: {}", e);
            eprintln!(
                "This is a critical error - the application cannot function without async support."
            );
            std::process::exit(1);
        }
    };

    // Try to load app icon
    let icon = load_app_icon();

    let mut viewport_builder = egui::ViewportBuilder::default()
        .with_inner_size([800.0, 600.0])
        .with_min_inner_size([400.0, 300.0])
        .with_app_id("com.bitvault.desktop"); // Set app ID for Wayland/Linux window managers

    if let Some(icon_data) = icon {
        eprintln!("Loaded app icon: {}x{}", icon_data.width, icon_data.height);
        viewport_builder = viewport_builder.with_icon(std::sync::Arc::new(icon_data));
    } else {
        eprintln!("No app icon found - window will use default icon");
        eprintln!("Looking for app.ico or app.png in resources directory");
    }

    let native_options = eframe::NativeOptions {
        viewport: viewport_builder,
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
