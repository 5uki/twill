use crate::domain::account::{
    AccountConnectionTestInput, AccountConnectionTestResult, AccountSummary, AddAccountInput,
};
use crate::domain::error::AppError;
use crate::infra::account_preflight::RuleBasedAccountConnectionTester;
use crate::infra::account_store::JsonFileAccountRepository;
use crate::services::account_service;

#[tauri::command]
pub fn list_accounts() -> Result<Vec<AccountSummary>, AppError> {
    let repository = JsonFileAccountRepository::from_default_path()?;

    account_service::list_accounts(&repository)
}

#[tauri::command]
pub fn add_account(input: AddAccountInput) -> Result<AccountSummary, AppError> {
    let repository = JsonFileAccountRepository::from_default_path()?;

    account_service::add_account(&repository, input)
}

#[tauri::command]
pub fn test_account_connection(
    input: AccountConnectionTestInput,
) -> Result<AccountConnectionTestResult, AppError> {
    let tester = RuleBasedAccountConnectionTester;

    account_service::test_account_connection(&tester, input)
}
