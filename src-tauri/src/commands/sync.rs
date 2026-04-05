use crate::domain::error::AppError;
use crate::domain::workspace::WorkspaceBootstrapSnapshot;
use crate::infra::account_store::JsonFileAccountRepository;
use crate::infra::workspace_store::JsonFileWorkspaceRepository;
use crate::infra::workspace_sync_source::SeededWorkspaceSyncSource;
use crate::services::workspace_service;

#[tauri::command]
pub async fn sync_workspace() -> Result<WorkspaceBootstrapSnapshot, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let account_repository = JsonFileAccountRepository::from_default_path()?;
        let workspace_repository = JsonFileWorkspaceRepository::from_default_path()?;
        let sync_source = SeededWorkspaceSyncSource;

        workspace_service::sync_workspace(&account_repository, &workspace_repository, &sync_source)
    })
    .await
    .map_err(|error| AppError::Storage {
        message: format!("等待工作台同步任务失败: {error}"),
    })?
}
