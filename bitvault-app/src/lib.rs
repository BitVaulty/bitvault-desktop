use anyhow::Result;
use std::fs;
use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn save_encrypted_seed(app: tauri::AppHandle, encrypted_seed: String) -> String {
    println!("save_encrypted_seed called");

    let result = (|| -> Result<(), String> {
        let cfg_path = app.path().config_dir().map_err(|e| e.to_string())?;
        let file_path = cfg_path.join("seed.encrypt");
        fs::write(file_path, encrypted_seed).map_err(|e| e.to_string())?;
        Ok(())
    })();

    match result {
        Ok(_) => "OK".to_string(),
        Err(e) => format!("ERR {}", e),
    }
}

#[tauri::command]
fn load_encrypted_seed(app: tauri::AppHandle) -> String {
    println!("load_encrypted_seed called");

    let result = (|| -> Result<String, String> {
        let cfg_path = app.path().config_dir().map_err(|e| e.to_string())?;
        let file_path = cfg_path.join("seed.encrypt");
        let encrypted_seed = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
        Ok(encrypted_seed)
    })();

    match result {
        Ok(encrypted_seed) => encrypted_seed,
        Err(e) => format!("ERR {}", e),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut ctx = tauri::generate_context!();
    let mut builder = tauri::Builder::default();

    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    {
        builder = builder.plugin(tauri_plugin_theme::init(ctx.config_mut()));
    }

    builder
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            save_encrypted_seed,
            load_encrypted_seed
        ])
        .run(ctx)
        .expect("error while running tauri application");
}
