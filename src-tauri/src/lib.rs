mod commands;
mod domain;
mod infra;
mod services;

pub mod cli;

use commands::account::{add_account, list_accounts, test_account_connection};
use commands::compose::{prepare_compose_draft, send_message};
use commands::sync::sync_workspace;
use commands::workspace::{
    apply_workspace_message_action, confirm_workspace_site, list_workspace_messages,
    load_workspace_bootstrap, open_workspace_message, open_workspace_message_original,
    read_workspace_message, resolve_workspace_site_context, update_workspace_message_read_state,
    update_workspace_message_status,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_secure_storage::init())
        .invoke_handler(tauri::generate_handler![
            load_workspace_bootstrap,
            list_workspace_messages,
            read_workspace_message,
            open_workspace_message,
            open_workspace_message_original,
            resolve_workspace_site_context,
            confirm_workspace_site,
            apply_workspace_message_action,
            update_workspace_message_read_state,
            update_workspace_message_status,
            sync_workspace,
            list_accounts,
            add_account,
            test_account_connection,
            prepare_compose_draft,
            send_message
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
