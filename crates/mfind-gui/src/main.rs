//! mfind-gui: Tauri-based GUI for mfind

mod commands;

use commands::*;
use mfind_core::{IndexEngine, index::engine::IndexConfig};
use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
use tauri_plugin_single_instance::init;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(GuiState {
            engine: Arc::new(RwLock::new(
                IndexEngine::new(IndexConfig::default()).expect("Failed to create IndexEngine")
            )),
        })
        .plugin(init(|app, argv, cwd| {
            // When a second instance is launched, focus the existing window
            println!("a single instance is already running: argv={argv:?}, cwd={cwd:?}");
            let _ = app.get_webview_window("main").map(|w| {
                let _ = w.show();
                let _ = w.set_focus();
            });
        }))
        .setup(|app| {
            // Create system tray menu
            let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide", "隐藏窗口", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出 mfind", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &hide, &quit])?;

            // Create tray icon with menu
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("mfind - 快速文件搜索")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            search,
            get_stats,
            build_index,
            toggle_window,
            hide_window,
            show_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running mfind-gui");
}
