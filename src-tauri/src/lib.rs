mod commands;
mod domain;
mod infra;
mod services;

pub mod cli;

use commands::workspace::load_workspace_bootstrap;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![load_workspace_bootstrap])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
