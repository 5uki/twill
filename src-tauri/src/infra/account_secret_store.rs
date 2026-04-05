use crate::domain::error::AppError;
use crate::services::account_service::AccountSecretStore;
use tauri::{AppHandle, Runtime};
use tauri_plugin_secure_storage::{OptionsRequest, SecureStorageExt};

const DEFAULT_SECRET_SERVICE_NAME: &str = "twill";
const DEFAULT_SECRET_KEY_PREFIX: &str = "accounts";

pub struct KeyringAccountSecretStore {
    service_name: String,
}

impl Default for KeyringAccountSecretStore {
    fn default() -> Self {
        Self::from_default_service_name()
    }
}

impl KeyringAccountSecretStore {
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }

    pub fn from_default_service_name() -> Self {
        let service_name = std::env::var("TWILL_SECRET_SERVICE")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_SECRET_SERVICE_NAME.to_string());

        Self::new(service_name)
    }

    fn entry(&self, account_id: &str) -> Result<keyring::Entry, AppError> {
        keyring::Entry::new(&self.service_name, &build_secret_key(account_id)).map_err(|error| {
            AppError::Storage {
                message: format!("创建系统安全存储条目失败: {error}"),
            }
        })
    }
}

impl AccountSecretStore for KeyringAccountSecretStore {
    fn save_secret(&self, account_id: &str, secret: &str) -> Result<(), AppError> {
        self.entry(account_id)?
            .set_password(secret)
            .map_err(|error| AppError::Storage {
                message: format!("写入系统安全存储失败: {error}"),
            })
    }

    fn delete_secret(&self, account_id: &str) -> Result<(), AppError> {
        match self.entry(account_id)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(AppError::Storage {
                message: format!("删除系统安全存储失败: {error}"),
            }),
        }
    }

    fn has_secret(&self, account_id: &str) -> Result<bool, AppError> {
        match self.entry(account_id)?.get_password() {
            Ok(_) => Ok(true),
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(error) => Err(AppError::Storage {
                message: format!("读取系统安全存储失败: {error}"),
            }),
        }
    }
}

pub struct TauriSecureStorageAccountSecretStore<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime> TauriSecureStorageAccountSecretStore<R> {
    pub fn new(app: AppHandle<R>) -> Self {
        Self { app }
    }

    fn payload(&self, account_id: &str, data: Option<&str>) -> OptionsRequest {
        OptionsRequest {
            prefixed_key: Some(build_secret_key(account_id)),
            data: data.map(ToOwned::to_owned),
            sync: None,
            keychain_access: None,
        }
    }

    fn map_storage_error(action: &str, error: tauri_plugin_secure_storage::Error) -> AppError {
        AppError::Storage {
            message: format!("{action}失败: {error}"),
        }
    }

    fn is_missing_entry(error: &tauri_plugin_secure_storage::Error) -> bool {
        error.to_string().to_ascii_lowercase().contains("not found")
    }
}

impl<R: Runtime> AccountSecretStore for TauriSecureStorageAccountSecretStore<R> {
    fn save_secret(&self, account_id: &str, secret: &str) -> Result<(), AppError> {
        self.app
            .secure_storage()
            .set_item(self.app.clone(), self.payload(account_id, Some(secret)))
            .map(|_| ())
            .map_err(|error| Self::map_storage_error("写入系统安全存储", error))
    }

    fn delete_secret(&self, account_id: &str) -> Result<(), AppError> {
        match self
            .app
            .secure_storage()
            .remove_item(self.app.clone(), self.payload(account_id, None))
        {
            Ok(_) => Ok(()),
            Err(error) if Self::is_missing_entry(&error) => Ok(()),
            Err(error) => Err(Self::map_storage_error("删除系统安全存储凭据", error)),
        }
    }

    fn has_secret(&self, account_id: &str) -> Result<bool, AppError> {
        match self
            .app
            .secure_storage()
            .get_item(self.app.clone(), self.payload(account_id, None))
        {
            Ok(response) => Ok(response.data.is_some()),
            Err(error) if Self::is_missing_entry(&error) => Ok(false),
            Err(error) => Err(Self::map_storage_error("读取系统安全存储凭据", error)),
        }
    }
}

fn build_secret_key(account_id: &str) -> String {
    format!("{DEFAULT_SECRET_KEY_PREFIX}/{account_id}")
}

#[cfg(test)]
pub(crate) fn keyring_secret_key(account_id: &str) -> String {
    build_secret_key(account_id)
}

#[cfg(test)]
pub(crate) fn default_secret_service_name() -> &'static str {
    DEFAULT_SECRET_SERVICE_NAME
}

#[cfg(test)]
mod tests {
    use super::{KeyringAccountSecretStore, default_secret_service_name, keyring_secret_key};
    use crate::services::account_service::AccountSecretStore;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn key_format_matches_tauri_plugin_contract() {
        assert_eq!(default_secret_service_name(), "twill");
        assert_eq!(
            keyring_secret_key("acct_1_primary"),
            "accounts/acct_1_primary"
        );
    }

    #[test]
    fn saves_checks_and_deletes_secret_in_system_store() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("系统时间应晚于 epoch")
            .as_nanos();
        let service_name = format!("twill.test.{suffix}");
        let account_id = format!("acct_test_{suffix}");
        let store = KeyringAccountSecretStore::new(service_name);

        store
            .delete_secret(&account_id)
            .expect("测试前清理系统凭据应成功");
        assert!(
            !store
                .has_secret(&account_id)
                .expect("读取初始凭据状态应成功"),
            "新建测试条目默认不应存在凭据"
        );

        store
            .save_secret(&account_id, "app-password")
            .expect("写入系统安全存储应成功");
        assert!(
            store
                .has_secret(&account_id)
                .expect("写入后读取凭据状态应成功"),
            "写入后应能看到已存储状态"
        );

        store
            .delete_secret(&account_id)
            .expect("删除测试系统凭据应成功");
        assert!(
            !store
                .has_secret(&account_id)
                .expect("删除后读取凭据状态应成功"),
            "删除后凭据状态应恢复为缺失"
        );
    }
}
