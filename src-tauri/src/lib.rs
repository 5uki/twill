mod commands;
mod domain;
mod infra;
mod services;

pub mod cli;

use commands::account::{add_account, list_accounts, test_account_connection};
use commands::sync::sync_workspace;
use commands::workspace::load_workspace_bootstrap;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_secure_storage::init())
        .invoke_handler(tauri::generate_handler![
            load_workspace_bootstrap,
            sync_workspace,
            list_accounts,
            add_account,
            test_account_connection
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
