use crate::domain::error::AppError;
use crate::domain::workspace::WorkspaceBootstrapSnapshot;
use crate::services::workspace_service;

#[tauri::command]
pub fn load_workspace_bootstrap() -> Result<WorkspaceBootstrapSnapshot, AppError> {
    Ok(workspace_service::load_workspace_bootstrap())
}
