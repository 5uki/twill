use crate::domain::error::AppError;
use crate::domain::workspace::{
    MessageReadState, MessageStatus, WorkspaceBootstrapSnapshot, WorkspaceMessageAction,
    WorkspaceMessageActionResult, WorkspaceMessageDetail, WorkspaceMessageItem,
    WorkspaceMessageOpenResult, WorkspaceMessageOriginalOpenResult, WorkspaceSiteContextResolution,
};
use crate::infra::workspace_store::JsonFileWorkspaceRepository;
use crate::services::workspace_service::{self, WorkspaceMessageFilter};

#[tauri::command]
pub fn load_workspace_bootstrap() -> Result<WorkspaceBootstrapSnapshot, AppError> {
    let repository = JsonFileWorkspaceRepository::from_default_path()?;

    workspace_service::load_workspace_bootstrap(&repository)
}

#[tauri::command]
pub fn list_workspace_messages(
    filter: Option<WorkspaceMessageFilter>,
) -> Result<Vec<WorkspaceMessageItem>, AppError> {
    let repository = JsonFileWorkspaceRepository::from_default_path()?;

    workspace_service::list_workspace_messages(&repository, &filter.unwrap_or_default())
}

#[tauri::command]
pub fn read_workspace_message(message_id: String) -> Result<WorkspaceMessageDetail, AppError> {
    let repository = JsonFileWorkspaceRepository::from_default_path()?;

    workspace_service::read_workspace_message(&repository, &message_id)
}

#[tauri::command]
pub fn open_workspace_message(message_id: String) -> Result<WorkspaceMessageOpenResult, AppError> {
    let repository = JsonFileWorkspaceRepository::from_default_path()?;

    workspace_service::open_workspace_message(&repository, &message_id)
}

#[tauri::command]
pub fn open_workspace_message_original(
    message_id: String,
) -> Result<WorkspaceMessageOriginalOpenResult, AppError> {
    let repository = JsonFileWorkspaceRepository::from_default_path()?;

    workspace_service::open_workspace_message_original(&repository, &message_id)
}

#[tauri::command]
pub fn resolve_workspace_site_context(
    input: String,
) -> Result<WorkspaceSiteContextResolution, AppError> {
    let repository = JsonFileWorkspaceRepository::from_default_path()?;

    workspace_service::resolve_workspace_site_context(&repository, &input)
}

#[tauri::command]
pub fn confirm_workspace_site(
    input: String,
    label: Option<String>,
) -> Result<WorkspaceBootstrapSnapshot, AppError> {
    let repository = JsonFileWorkspaceRepository::from_default_path()?;

    workspace_service::confirm_workspace_site(&repository, &input, label.as_deref())
}

#[tauri::command]
pub fn apply_workspace_message_action(
    message_id: String,
    action: WorkspaceMessageAction,
) -> Result<WorkspaceMessageActionResult, AppError> {
    let repository = JsonFileWorkspaceRepository::from_default_path()?;

    workspace_service::apply_workspace_message_action(&repository, &message_id, action)
}

#[tauri::command]
pub fn update_workspace_message_status(
    message_id: String,
    status: MessageStatus,
) -> Result<WorkspaceBootstrapSnapshot, AppError> {
    let repository = JsonFileWorkspaceRepository::from_default_path()?;

    workspace_service::update_workspace_message_status(&repository, &message_id, status)
}

#[tauri::command]
pub fn update_workspace_message_read_state(
    message_id: String,
    read_state: MessageReadState,
) -> Result<WorkspaceBootstrapSnapshot, AppError> {
    let repository = JsonFileWorkspaceRepository::from_default_path()?;

    workspace_service::update_workspace_message_read_state(&repository, &message_id, read_state)
}
