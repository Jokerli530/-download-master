// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod downloader;
mod task;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::add_task,
            commands::pause_task,
            commands::resume_task,
            commands::cancel_task,
            commands::get_tasks,
            commands::get_task,
            commands::set_speed_limit,
            commands::clear_completed,
            commands::select_save_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
