use crate::domain::account::{
    AccountConnectionTestInput, AccountConnectionTestResult, AccountSummary, AddAccountInput,
};
use crate::domain::error::AppError;
use crate::infra::account_preflight::LiveAccountConnectionTester;
use crate::infra::account_secret_store::TauriSecureStorageAccountSecretStore;
use crate::infra::account_store::JsonFileAccountRepository;
use crate::services::account_service;
use tauri::{AppHandle, Runtime};

#[tauri::command]
pub fn list_accounts<R: Runtime>(app: AppHandle<R>) -> Result<Vec<AccountSummary>, AppError> {
    let repository = JsonFileAccountRepository::from_default_path()?;
    let secret_store = TauriSecureStorageAccountSecretStore::new(app);

    account_service::list_accounts(&repository, &secret_store)
}

#[tauri::command]
pub fn add_account<R: Runtime>(
    app: AppHandle<R>,
    input: AddAccountInput,
) -> Result<AccountSummary, AppError> {
    let repository = JsonFileAccountRepository::from_default_path()?;
    let secret_store = TauriSecureStorageAccountSecretStore::new(app);

    account_service::add_account(&repository, &secret_store, input)
}

#[tauri::command]
pub fn test_account_connection(
    input: AccountConnectionTestInput,
) -> Result<AccountConnectionTestResult, AppError> {
    let tester = LiveAccountConnectionTester::default();

    account_service::test_account_connection(&tester, input)
}
