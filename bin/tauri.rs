use std::process::Command;

fn main() {
    // Run tauri from the right directory
    let status = Command::new("cargo")
        .args(["tauri", "--config", "./crates/bitvault-app/tauri.conf.json", "dev"])
        .current_dir(".")
        .status()
        .expect("Failed to execute command");

    std::process::exit(status.code().unwrap_or(1));
}