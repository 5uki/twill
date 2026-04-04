use crate::domain::account::AccountSummary;
use crate::domain::error::AppError;
use crate::services::account_service::AccountRepository;
use std::fs;
use std::path::{Path, PathBuf};

pub struct JsonFileAccountRepository {
    path: PathBuf,
}

impl JsonFileAccountRepository {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn from_default_path() -> Result<Self, AppError> {
        Ok(Self::new(default_account_store_path()))
    }

    fn read_accounts(&self) -> Result<Vec<AccountSummary>, AppError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.path).map_err(|error| AppError::Storage {
            message: format!("读取账户存储失败: {error}"),
        })?;

        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        serde_json::from_str::<Vec<AccountSummary>>(&content).map_err(|error| {
            AppError::Serialization {
                message: format!("解析账户存储失败: {error}"),
            }
        })
    }

    fn write_accounts(&self, accounts: &[AccountSummary]) -> Result<(), AppError> {
        ensure_parent_directory(&self.path)?;

        let content =
            serde_json::to_string_pretty(accounts).map_err(|error| AppError::Serialization {
                message: format!("序列化账户存储失败: {error}"),
            })?;

        fs::write(&self.path, content).map_err(|error| AppError::Storage {
            message: format!("写入账户存储失败: {error}"),
        })
    }
}

impl AccountRepository for JsonFileAccountRepository {
    fn list_accounts(&self) -> Result<Vec<AccountSummary>, AppError> {
        self.read_accounts()
    }

    fn save_account(&self, account: &AccountSummary) -> Result<(), AppError> {
        let mut accounts = self.read_accounts()?;
        accounts.push(account.clone());
        self.write_accounts(&accounts)
    }
}

pub fn default_account_store_path() -> PathBuf {
    if let Some(path) = std::env::var_os("TWILL_ACCOUNT_STORE") {
        return PathBuf::from(path);
    }

    std::env::temp_dir().join("twill-dev").join("accounts.json")
}

fn ensure_parent_directory(path: &Path) -> Result<(), AppError> {
    match path.parent() {
        Some(parent) => fs::create_dir_all(parent).map_err(|error| AppError::Storage {
            message: format!("创建账户存储目录失败: {error}"),
        }),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::JsonFileAccountRepository;
    use crate::domain::account::{AccountSummary, MailSecurity, MailServerConfig};
    use crate::services::account_service::AccountRepository;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn saves_and_reads_accounts_from_json_file() {
        let repository = JsonFileAccountRepository::new(unique_test_file_path());
        let account = AccountSummary {
            id: "acct_1_primary".to_string(),
            display_name: "Primary".to_string(),
            email: "primary@example.com".to_string(),
            login: "primary@example.com".to_string(),
            imap: MailServerConfig {
                host: "imap.example.com".to_string(),
                port: 993,
                security: MailSecurity::Tls,
            },
            smtp: MailServerConfig {
                host: "smtp.example.com".to_string(),
                port: 587,
                security: MailSecurity::StartTls,
            },
        };

        repository.save_account(&account).expect("保存账户应成功");
        let accounts = repository.list_accounts().expect("读取账户应成功");

        assert_eq!(accounts, vec![account]);
    }

    fn unique_test_file_path() -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("系统时间应晚于 epoch")
            .as_nanos();

        std::env::temp_dir()
            .join("twill-tests")
            .join(format!("accounts-{suffix}.json"))
    }
}
