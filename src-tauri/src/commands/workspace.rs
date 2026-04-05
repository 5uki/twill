use crate::domain::error::AppError;
use crate::domain::workspace::WorkspaceBootstrapSnapshot;
use crate::infra::workspace_store::JsonFileWorkspaceRepository;
use crate::services::workspace_service;

#[tauri::command]
pub fn load_workspace_bootstrap() -> Result<WorkspaceBootstrapSnapshot, AppError> {
    let repository = JsonFileWorkspaceRepository::from_default_path()?;

    workspace_service::load_workspace_bootstrap(&repository)
}
