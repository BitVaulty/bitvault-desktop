// VaultService in Arc is used from main thread only; Send/Sync not required for egui
#![allow(clippy::arc_with_non_send_sync)]
// Various API surfaces and future-use code
#![allow(dead_code)]

mod app;
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
        // Relative to project root
        PathBuf::from("resources/app_64.png"),
        PathBuf::from("resources/app_128.png"),
        PathBuf::from("resources/app.ico"),
        PathBuf::from("resources/app.png"),
    ];

    // Also try in the executable directory
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            possible_paths.push(exe_dir.join("resources/app_64.png"));
            possible_paths.push(exe_dir.join("resources/app_128.png"));
            possible_paths.push(exe_dir.join("resources/app.ico"));
            possible_paths.push(exe_dir.join("resources/app.png"));

            // If we're in target/release, go up to find resources
            let mut current = exe_dir;
            while let Some(parent) = current.parent() {
                let resources_ico = parent.join("resources/app.ico");
                if resources_ico.exists() {
                    possible_paths.push(parent.join("resources/app_64.png"));
                    possible_paths.push(parent.join("resources/app_128.png"));
                    possible_paths.push(resources_ico.clone());
                    possible_paths.push(parent.join("resources/app.png"));
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

/// Check if current version meets minimum required (semantic versioning)
fn is_version_at_least(current: &str, minimum: &str) -> bool {
    let parse = |s: &str| -> Vec<u32> {
        s.split('.')
            .filter_map(|p| p.parse::<u32>().ok())
            .collect::<Vec<_>>()
    };
    let current_parts = parse(current);
    let minimum_parts = parse(minimum);
    for i in 0..minimum_parts.len().max(current_parts.len()) {
        let c = current_parts.get(i).copied().unwrap_or(0);
        let m = minimum_parts.get(i).copied().unwrap_or(0);
        if c > m {
            return true;
        }
        if c < m {
            return false;
        }
    }
    true // Equal
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

    // Check app version against remote config (don't block on network errors)
    let is_update_required = rt.block_on(async {
        // Skip version check if offline (avoid long timeout)
        if !crate::services::network_check::check_connectivity().await {
            log::info!("Offline - skipping version check");
            return false;
        }
        let cs = bitvault_common::ConvenienceService::new(None);
        match cs.get_remote_config().await {
            Ok(config) => {
                let current = env!("CARGO_PKG_VERSION");
                let min_required = &config.app.minimum_version;
                if !is_version_at_least(current, min_required) {
                    log::warn!(
                        "App version {} is below minimum required {}",
                        current,
                        min_required
                    );
                    true
                } else {
                    false
                }
            }
            Err(e) => {
                log::warn!("Failed to fetch remote config, allowing app to run: {}", e);
                false
            }
        }
    });

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
            let mut app = app::BitVaultApp::new(cc, is_update_required);
            app.set_runtime(rt);
            Box::new(app)
        }),
    ) {
        eprintln!("Failed to start application: {}", e);
        std::process::exit(1);
    }
}
