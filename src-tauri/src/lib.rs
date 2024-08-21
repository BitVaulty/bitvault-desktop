// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let ctx = tauri::generate_context!();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        // .plugin(tauri_plugin_theme::init(ctx.config_mut()))
        .invoke_handler(tauri::generate_handler![greet])
        .run(ctx)
        .expect("error while running tauri application");
}
