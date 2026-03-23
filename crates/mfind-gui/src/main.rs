//! mfind-gui: Tauri-based GUI for mfind

mod commands;

use commands::*;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![search, get_stats, build_index])
        .run(tauri::generate_context!())
        .expect("error while running mfind-gui");
}
