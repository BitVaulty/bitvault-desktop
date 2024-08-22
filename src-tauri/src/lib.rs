// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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
        .invoke_handler(tauri::generate_handler![greet])
        .run(ctx)
        .expect("error while running tauri application");
}
