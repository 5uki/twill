use crate::domain::compose::{
    PrepareComposeInput, PreparedComposeDraft, SendMessageInput, SendMessageResult,
};
use crate::domain::error::AppError;
use crate::infra::account_secret_store::TauriSecureStorageAccountSecretStore;
use crate::infra::account_store::JsonFileAccountRepository;
use crate::infra::compose_delivery::LiveComposeDeliveryClient;
use crate::infra::workspace_store::JsonFileWorkspaceRepository;
use crate::services::compose_service;
use tauri::{AppHandle, Runtime};

#[tauri::command]
pub fn prepare_compose_draft(input: PrepareComposeInput) -> Result<PreparedComposeDraft, AppError> {
    let repository = JsonFileWorkspaceRepository::from_default_path()?;

    compose_service::prepare_compose_draft(&repository, input)
}

#[tauri::command]
pub async fn send_message<R: Runtime>(
    app: AppHandle<R>,
    input: SendMessageInput,
) -> Result<SendMessageResult, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let repository = JsonFileAccountRepository::from_default_path()?;
        let secret_store = TauriSecureStorageAccountSecretStore::new(app);
        let delivery_client = LiveComposeDeliveryClient::default();

        compose_service::send_message(&repository, &secret_store, &delivery_client, input)
    })
    .await
    .map_err(|error| AppError::Storage {
        message: format!("等待发送任务失败: {error}"),
    })?
}
