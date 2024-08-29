use anyhow::Result;
use bip39::{Language, Mnemonic};
use zeroize::Zeroize;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
fn new_seed_handler() -> Result<String> {
    let mut entropy = [0u8; 16];
    getrandom::getrandom(&mut entropy)?;
    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)?;
    entropy.zeroize();
    Ok(mnemonic.to_string())
}

#[tauri::command]
fn new_seed() -> String {
    println!("new_seed called");
    new_seed_handler().unwrap_or_else(|e| e.to_string())
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
        .invoke_handler(tauri::generate_handler![new_seed])
        .run(ctx)
        .expect("error while running tauri application");
}
