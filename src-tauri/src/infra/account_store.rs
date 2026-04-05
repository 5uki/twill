use crate::domain::account::AccountSummary;
use crate::domain::error::AppError;
use crate::services::account_service::AccountRepository;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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

        write_atomic(&self.path, &content)
    }

    fn lock_path(&self) -> PathBuf {
        let mut lock_name = OsString::from(self.path.as_os_str());
        lock_name.push(".lock");
        PathBuf::from(lock_name)
    }

    fn acquire_lock(&self) -> Result<FileLockGuard, AppError> {
        acquire_file_lock(&self.lock_path())
    }
}

impl AccountRepository for JsonFileAccountRepository {
    fn list_accounts(&self) -> Result<Vec<AccountSummary>, AppError> {
        let _guard = self.acquire_lock()?;

        self.read_accounts()
    }

    fn save_account(&self, account: &AccountSummary) -> Result<(), AppError> {
        let _guard = self.acquire_lock()?;
        let mut accounts = self.read_accounts()?;

        if accounts
            .iter()
            .any(|existing| existing.email.eq_ignore_ascii_case(&account.email))
        {
            return Err(AppError::Validation {
                field: "email".to_string(),
                message: "该邮箱地址已经存在".to_string(),
            });
        }

        accounts.push(account.clone());
        self.write_accounts(&accounts)
    }

    fn delete_account(&self, account_id: &str) -> Result<(), AppError> {
        let _guard = self.acquire_lock()?;
        let mut accounts = self.read_accounts()?;
        let original_len = accounts.len();
        accounts.retain(|account| account.id != account_id);

        if accounts.len() == original_len {
            return Ok(());
        }

        self.write_accounts(&accounts)
    }
}

pub fn default_account_store_path() -> PathBuf {
    if let Some(path) = std::env::var_os("TWILL_ACCOUNT_STORE") {
        return PathBuf::from(path);
    }

    default_data_root().join("Twill").join("accounts.json")
}

fn ensure_parent_directory(path: &Path) -> Result<(), AppError> {
    match path.parent() {
        Some(parent) => fs::create_dir_all(parent).map_err(|error| AppError::Storage {
            message: format!("创建账户存储目录失败: {error}"),
        }),
        None => Ok(()),
    }
}

fn default_data_root() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("LOCALAPPDATA")
            .or_else(|| std::env::var_os("APPDATA"))
            .map(PathBuf::from)
            .unwrap_or_else(fallback_home_dir)
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        fallback_home_dir()
            .join("Library")
            .join("Application Support")
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "ios")))]
    {
        if let Some(path) = std::env::var_os("XDG_DATA_HOME") {
            PathBuf::from(path)
        } else {
            fallback_home_dir().join(".local").join("share")
        }
    }
}

fn fallback_home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn acquire_file_lock(path: &Path) -> Result<FileLockGuard, AppError> {
    ensure_parent_directory(path)?;

    let deadline = Instant::now() + Duration::from_secs(5);

    loop {
        match OpenOptions::new().write(true).create_new(true).open(path) {
            Ok(file) => {
                return Ok(FileLockGuard {
                    path: path.to_path_buf(),
                    file: Some(file),
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if Instant::now() >= deadline {
                    return Err(AppError::Storage {
                        message: format!("等待账户存储锁超时: {}", path.display()),
                    });
                }

                thread::sleep(Duration::from_millis(25));
            }
            Err(error) => {
                return Err(AppError::Storage {
                    message: format!("创建账户存储锁失败: {error}"),
                });
            }
        }
    }
}

struct FileLockGuard {
    path: PathBuf,
    file: Option<File>,
}

impl Drop for FileLockGuard {
    fn drop(&mut self) {
        self.file.take();
        let _ = fs::remove_file(&self.path);
    }
}

fn unique_temp_file_path(path: &Path) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("系统时间应晚于 epoch")
        .as_nanos();
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "accounts.json".to_string());

    path.with_file_name(format!("{file_name}.{suffix}.tmp"))
}

fn write_atomic(path: &Path, content: &str) -> Result<(), AppError> {
    let temp_path = unique_temp_file_path(path);
    let mut temp_file = File::create(&temp_path).map_err(|error| AppError::Storage {
        message: format!("创建账户临时存储失败: {error}"),
    })?;

    temp_file
        .write_all(content.as_bytes())
        .map_err(|error| AppError::Storage {
            message: format!("写入账户临时存储失败: {error}"),
        })?;
    temp_file.sync_all().map_err(|error| AppError::Storage {
        message: format!("同步账户临时存储失败: {error}"),
    })?;
    drop(temp_file);

    replace_file(&temp_path, path).map_err(|error| AppError::Storage {
        message: format!("替换账户存储文件失败: {error}"),
    })?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn replace_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    if !destination.exists() {
        return fs::rename(source, destination);
    }

    windows_replace_file(source, destination)
}

#[cfg(not(target_os = "windows"))]
fn replace_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::rename(source, destination)
}

#[cfg(target_os = "windows")]
fn windows_replace_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

    const REPLACEFILE_WRITE_THROUGH: u32 = 0x0000_0001;
    const REPLACEFILE_IGNORE_MERGE_ERRORS: u32 = 0x0000_0002;

    #[link(name = "Kernel32")]
    unsafe extern "system" {
        fn ReplaceFileW(
            replaced_file_name: *const u16,
            replacement_file_name: *const u16,
            backup_file_name: *const u16,
            replace_flags: u32,
            exclude: *mut core::ffi::c_void,
            reserved: *mut core::ffi::c_void,
        ) -> i32;
    }

    let destination_wide = destination
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let source_wide = source
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();

    let replaced = unsafe {
        ReplaceFileW(
            destination_wide.as_ptr(),
            source_wide.as_ptr(),
            std::ptr::null(),
            REPLACEFILE_WRITE_THROUGH | REPLACEFILE_IGNORE_MERGE_ERRORS,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };

    if replaced == 0 {
        return Err(std::io::Error::last_os_error());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{JsonFileAccountRepository, default_account_store_path};
    use crate::domain::account::{
        AccountCredentialState, AccountSummary, MailSecurity, MailServerConfig,
    };
    use crate::domain::error::AppError;
    use crate::services::account_service::AccountRepository;
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn saves_and_reads_accounts_from_json_file() {
        let repository = JsonFileAccountRepository::new(unique_test_file_path());
        let account = sample_account("acct_primary-example-com", "primary@example.com");

        repository.save_account(&account).expect("保存账户应成功");
        let accounts = repository.list_accounts().expect("读取账户应成功");

        assert_eq!(accounts, vec![account]);
    }

    #[test]
    fn uses_persistent_default_store_path_instead_of_temp_directory() {
        let default_path = default_account_store_path();

        assert!(
            !default_path.starts_with(std::env::temp_dir()),
            "默认账户存储路径不应落在系统临时目录: {}",
            default_path.display()
        );
    }

    #[test]
    fn rejects_duplicate_email_inside_repository_boundary() {
        let repository = JsonFileAccountRepository::new(unique_test_file_path());
        let first = sample_account("acct_primary-example-com", "primary@example.com");
        let duplicate = sample_account("acct_duplicate", "primary@example.com");

        repository.save_account(&first).expect("首次保存应成功");
        let error = repository
            .save_account(&duplicate)
            .expect_err("重复邮箱必须在仓库层拒绝");

        assert_eq!(
            error,
            AppError::Validation {
                field: "email".to_string(),
                message: "该邮箱地址已经存在".to_string(),
            }
        );
    }

    #[test]
    fn keeps_all_accounts_when_multiple_threads_save_at_once() {
        let repository = Arc::new(JsonFileAccountRepository::new(unique_test_file_path()));
        let barrier = Arc::new(Barrier::new(3));
        let mut handles = Vec::new();

        for (id, email) in [
            ("acct_primary-example-com", "primary@example.com"),
            ("acct_work-example-com", "work@example.com"),
        ] {
            let repository = Arc::clone(&repository);
            let barrier = Arc::clone(&barrier);
            let account = sample_account(id, email);

            handles.push(thread::spawn(move || {
                barrier.wait();
                repository.save_account(&account)
            }));
        }

        barrier.wait();

        for handle in handles {
            handle
                .join()
                .expect("并发保存线程不应 panic")
                .expect("并发保存应成功");
        }

        let accounts = repository.list_accounts().expect("读取账户应成功");

        assert_eq!(accounts.len(), 2);
    }

    fn sample_account(id: &str, email: &str) -> AccountSummary {
        AccountSummary {
            id: id.to_string(),
            display_name: "Primary".to_string(),
            email: email.to_string(),
            login: email.to_string(),
            credential_state: AccountCredentialState::Stored,
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
        }
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
