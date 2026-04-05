use crate::domain::account::{
    AccountConnectionCheckTarget, AccountConnectionStatus, AccountConnectionTestInput,
    AccountConnectionTestResult, AccountCredentialState, AddAccountInput, MailSecurity,
    MailServerConfig,
};
use crate::domain::error::AppError;
use crate::infra::account_preflight::LiveAccountConnectionTester;
use crate::infra::account_secret_store::KeyringAccountSecretStore;
use crate::infra::account_store::JsonFileAccountRepository;
use crate::services::account_service::{
    self, AccountConnectionTester, AccountRepository, AccountSecretStore,
};
use crate::services::workspace_service;
use std::collections::BTreeMap;
use std::path::PathBuf;

enum OutputFormat {
    Text,
    Json,
}

pub fn run_from_env() -> Result<String, AppError> {
    run_with_args(std::env::args().skip(1))
}

pub fn run_with_args<I, S>(args: I) -> Result<String, AppError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    run_with_args_and_store_path(args, default_store_path())
}

fn run_with_args_and_store_path<I, S>(args: I, store_path: PathBuf) -> Result<String, AppError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let repository = JsonFileAccountRepository::new(store_path);
    let secret_store = KeyringAccountSecretStore::from_default_service_name();
    let tester = LiveAccountConnectionTester::default();

    run_with_args_and_dependencies(args, &repository, &secret_store, &tester)
}

fn run_with_args_and_dependencies<I, S, R, SecretStore, Tester>(
    args: I,
    repository: &R,
    secret_store: &SecretStore,
    tester: &Tester,
) -> Result<String, AppError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
    R: AccountRepository,
    SecretStore: AccountSecretStore,
    Tester: AccountConnectionTester,
{
    let args = args
        .into_iter()
        .map(|value| value.as_ref().to_string())
        .collect::<Vec<_>>();

    match args.as_slice() {
        [workspace, bootstrap] if workspace == "workspace" && bootstrap == "bootstrap" => {
            render_workspace_bootstrap(OutputFormat::Text)
        }
        [workspace, bootstrap, flag, value]
            if workspace == "workspace" && bootstrap == "bootstrap" && flag == "--format" =>
        {
            render_workspace_bootstrap(parse_output_format(value)?)
        }
        [account, list] if account == "account" && list == "list" => {
            render_account_list(OutputFormat::Text, repository, secret_store)
        }
        [account, list, rest @ ..] if account == "account" && list == "list" => {
            let mut flags = parse_flags(rest)?;
            let format = take_output_format(&mut flags)?;

            ensure_no_unknown_flags(&flags)?;
            render_account_list(format, repository, secret_store)
        }
        [account, add, rest @ ..] if account == "account" && add == "add" => {
            let mut flags = parse_flags(rest)?;
            let format = take_output_format(&mut flags)?;
            let input = parse_add_account_input(&mut flags)?;

            ensure_no_unknown_flags(&flags)?;
            render_add_account(format, repository, secret_store, input)
        }
        [account, test, rest @ ..] if account == "account" && test == "test" => {
            let mut flags = parse_flags(rest)?;
            let format = take_output_format(&mut flags)?;
            let input = parse_account_test_input(&mut flags)?;

            ensure_no_unknown_flags(&flags)?;
            render_account_test(format, tester, input)
        }
        _ => Err(AppError::InvalidCliArgs {
            message: concat!(
                "用法:\n",
                "  workspace bootstrap [--format text|json]\n",
                "  account list [--format text|json]\n",
                "  account add --name <name> --email <email> --login <login> --password <password> ",
                "--imap-host <host> --imap-port <port> --imap-security <none|start_tls|tls> ",
                "--smtp-host <host> --smtp-port <port> --smtp-security <none|start_tls|tls> ",
                "[--format text|json]\n",
                "  account test --name <name> --email <email> --login <login> ",
                "--imap-host <host> --imap-port <port> --imap-security <none|start_tls|tls> ",
                "--smtp-host <host> --smtp-port <port> --smtp-security <none|start_tls|tls> ",
                "[--format text|json]"
            )
            .to_string(),
        }),
    }
}

fn render_workspace_bootstrap(format: OutputFormat) -> Result<String, AppError> {
    let snapshot = workspace_service::load_workspace_bootstrap();

    match format {
        OutputFormat::Text => {
            let navigation = snapshot
                .navigation
                .iter()
                .map(|item| format!("- {} ({})", item.label, item.badge))
                .collect::<Vec<_>>()
                .join("\n");

            Ok(format!(
                "Twill workspace bootstrap\n默认视图: Recent verification\n生成时间: {}\n导航:\n{}\n当前选中: {}\n验证码: {}\n链接: {}",
                snapshot.generated_at,
                navigation,
                snapshot.selected_message.subject,
                snapshot
                    .selected_message
                    .extracted_code
                    .as_deref()
                    .unwrap_or("无"),
                snapshot
                    .selected_message
                    .verification_link
                    .as_deref()
                    .unwrap_or("无")
            ))
        }
        OutputFormat::Json => {
            serde_json::to_string_pretty(&snapshot).map_err(|error| AppError::Serialization {
                message: error.to_string(),
            })
        }
    }
}

fn render_account_list<R, SecretStore>(
    format: OutputFormat,
    repository: &R,
    secret_store: &SecretStore,
) -> Result<String, AppError>
where
    R: AccountRepository,
    SecretStore: AccountSecretStore,
{
    let accounts = account_service::list_accounts(repository, secret_store)?;

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&accounts).map_err(|error| AppError::Serialization {
                message: error.to_string(),
            })
        }
        OutputFormat::Text => {
            if accounts.is_empty() {
                Ok("当前没有已保存的账户配置。".to_string())
            } else {
                let lines = accounts
                    .iter()
                    .map(|account| {
                        format!(
                            "- {} <{}> | 凭据 {} | IMAP {}:{} {:?} | SMTP {}:{} {:?}",
                            account.display_name,
                            account.email,
                            format_credential_state(account.credential_state),
                            account.imap.host,
                            account.imap.port,
                            account.imap.security,
                            account.smtp.host,
                            account.smtp.port,
                            account.smtp.security
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                Ok(format!("已保存账户\n{lines}"))
            }
        }
    }
}

fn render_add_account<R, SecretStore>(
    format: OutputFormat,
    repository: &R,
    secret_store: &SecretStore,
    input: AddAccountInput,
) -> Result<String, AppError>
where
    R: AccountRepository,
    SecretStore: AccountSecretStore,
{
    let account = account_service::add_account(repository, secret_store, input)?;

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&account).map_err(|error| AppError::Serialization {
                message: error.to_string(),
            })
        }
        OutputFormat::Text => Ok(format!(
            "账户已保存\nID: {}\n名称: {}\n邮箱: {}\n凭据: {}\nIMAP: {}:{} ({:?})\nSMTP: {}:{} ({:?})",
            account.id,
            account.display_name,
            account.email,
            format_credential_state(account.credential_state),
            account.imap.host,
            account.imap.port,
            account.imap.security,
            account.smtp.host,
            account.smtp.port,
            account.smtp.security
        )),
    }
}

fn render_account_test<T>(
    format: OutputFormat,
    tester: &T,
    input: AccountConnectionTestInput,
) -> Result<String, AppError>
where
    T: AccountConnectionTester,
{
    let result = account_service::test_account_connection(tester, input)?;

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&result).map_err(|error| AppError::Serialization {
                message: error.to_string(),
            })
        }
        OutputFormat::Text => Ok(format_account_test_result(&result)),
    }
}

fn format_account_test_result(result: &AccountConnectionTestResult) -> String {
    let status = match result.status {
        AccountConnectionStatus::Passed => "通过",
        AccountConnectionStatus::Failed => "失败",
    };

    let checks = result
        .checks
        .iter()
        .map(|check| {
            let target = match check.target {
                AccountConnectionCheckTarget::Identity => "Identity",
                AccountConnectionCheckTarget::Imap => "IMAP",
                AccountConnectionCheckTarget::Smtp => "SMTP",
            };
            let check_status = match check.status {
                AccountConnectionStatus::Passed => "通过",
                AccountConnectionStatus::Failed => "失败",
            };

            format!("- {target}: {check_status} | {}", check.message)
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "账户连接实时探测\n状态: {status}\n结论: {}\n{checks}",
        result.summary
    )
}

fn format_credential_state(state: AccountCredentialState) -> &'static str {
    match state {
        AccountCredentialState::Missing => "缺失",
        AccountCredentialState::Stored => "已存储",
    }
}

fn parse_flags(args: &[String]) -> Result<BTreeMap<String, String>, AppError> {
    let mut flags = BTreeMap::new();
    let mut index = 0;

    while index < args.len() {
        let flag = &args[index];

        if !flag.starts_with("--") {
            return Err(AppError::InvalidCliArgs {
                message: format!("无法识别的位置参数: {flag}"),
            });
        }

        let value = args
            .get(index + 1)
            .ok_or_else(|| AppError::InvalidCliArgs {
                message: format!("参数 {flag} 缺少取值"),
            })?;

        if value.starts_with("--") {
            return Err(AppError::InvalidCliArgs {
                message: format!("参数 {flag} 缺少取值"),
            });
        }

        flags.insert(flag.clone(), value.clone());
        index += 2;
    }

    Ok(flags)
}

fn take_output_format(flags: &mut BTreeMap<String, String>) -> Result<OutputFormat, AppError> {
    match flags.remove("--format") {
        Some(value) => parse_output_format(&value),
        None => Ok(OutputFormat::Text),
    }
}

fn parse_output_format(value: &str) -> Result<OutputFormat, AppError> {
    match value {
        "text" => Ok(OutputFormat::Text),
        "json" => Ok(OutputFormat::Json),
        other => Err(AppError::UnsupportedFormat {
            format: other.to_string(),
        }),
    }
}

fn parse_add_account_input(
    flags: &mut BTreeMap<String, String>,
) -> Result<AddAccountInput, AppError> {
    Ok(AddAccountInput {
        display_name: take_required_flag(flags, "--name")?,
        email: take_required_flag(flags, "--email")?,
        login: take_required_flag(flags, "--login")?,
        password: take_required_flag(flags, "--password")?,
        imap: MailServerConfig {
            host: take_required_flag(flags, "--imap-host")?,
            port: parse_port(&take_required_flag(flags, "--imap-port")?, "imap.port")?,
            security: parse_security(&take_required_flag(flags, "--imap-security")?)?,
        },
        smtp: MailServerConfig {
            host: take_required_flag(flags, "--smtp-host")?,
            port: parse_port(&take_required_flag(flags, "--smtp-port")?, "smtp.port")?,
            security: parse_security(&take_required_flag(flags, "--smtp-security")?)?,
        },
    })
}

fn parse_account_test_input(
    flags: &mut BTreeMap<String, String>,
) -> Result<AccountConnectionTestInput, AppError> {
    Ok(AccountConnectionTestInput {
        display_name: take_required_flag(flags, "--name")?,
        email: take_required_flag(flags, "--email")?,
        login: take_required_flag(flags, "--login")?,
        imap: MailServerConfig {
            host: take_required_flag(flags, "--imap-host")?,
            port: parse_port(&take_required_flag(flags, "--imap-port")?, "imap.port")?,
            security: parse_security(&take_required_flag(flags, "--imap-security")?)?,
        },
        smtp: MailServerConfig {
            host: take_required_flag(flags, "--smtp-host")?,
            port: parse_port(&take_required_flag(flags, "--smtp-port")?, "smtp.port")?,
            security: parse_security(&take_required_flag(flags, "--smtp-security")?)?,
        },
    })
}

fn take_required_flag(flags: &mut BTreeMap<String, String>, key: &str) -> Result<String, AppError> {
    flags.remove(key).ok_or_else(|| AppError::InvalidCliArgs {
        message: format!("缺少参数: {key}"),
    })
}

fn parse_port(value: &str, field: &str) -> Result<u16, AppError> {
    value.parse::<u16>().map_err(|_| AppError::Validation {
        field: field.to_string(),
        message: format!("端口必须是 1 到 65535 之间的整数，收到 {value}"),
    })
}

fn parse_security(value: &str) -> Result<MailSecurity, AppError> {
    match value {
        "none" => Ok(MailSecurity::None),
        "start_tls" => Ok(MailSecurity::StartTls),
        "tls" => Ok(MailSecurity::Tls),
        other => Err(AppError::Validation {
            field: "security".to_string(),
            message: format!("不支持的安全策略: {other}"),
        }),
    }
}

fn ensure_no_unknown_flags(flags: &BTreeMap<String, String>) -> Result<(), AppError> {
    if flags.is_empty() {
        Ok(())
    } else {
        let unknown = flags.keys().cloned().collect::<Vec<_>>().join(", ");

        Err(AppError::InvalidCliArgs {
            message: format!("存在未识别参数: {unknown}"),
        })
    }
}

fn default_store_path() -> PathBuf {
    crate::infra::account_store::default_account_store_path()
}

#[cfg(test)]
mod tests {
    use super::run_with_args_and_dependencies;
    use crate::domain::error::AppError;
    use crate::infra::account_preflight::LiveAccountConnectionTester;
    use crate::infra::account_store::JsonFileAccountRepository;
    use crate::services::account_service::AccountSecretStore;
    use serde_json::Value;
    use std::cell::RefCell;
    use std::collections::BTreeSet;
    use std::fs;
    use std::net::TcpListener;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Default)]
    struct InMemorySecretStore {
        stored_accounts: RefCell<BTreeSet<String>>,
    }

    impl AccountSecretStore for InMemorySecretStore {
        fn save_secret(&self, account_id: &str, _secret: &str) -> Result<(), AppError> {
            self.stored_accounts
                .borrow_mut()
                .insert(account_id.to_string());

            Ok(())
        }

        fn delete_secret(&self, account_id: &str) -> Result<(), AppError> {
            self.stored_accounts.borrow_mut().remove(account_id);
            Ok(())
        }

        fn has_secret(&self, account_id: &str) -> Result<bool, AppError> {
            Ok(self.stored_accounts.borrow().contains(account_id))
        }
    }

    #[test]
    fn defaults_to_text_output_for_workspace_bootstrap() {
        let output = run_with_args_and_test_store(["workspace", "bootstrap"], unique_store_path())
            .expect("命令应执行成功");

        assert!(
            output.contains("Recent verification"),
            "文本输出至少要包含默认工作台视图"
        );
    }

    #[test]
    fn persists_account_between_add_and_list() {
        let store_path = unique_store_path();
        let repository = JsonFileAccountRepository::new(store_path.clone());
        let secret_store = InMemorySecretStore::default();
        let tester = LiveAccountConnectionTester::default();

        run_with_args_and_dependencies(
            [
                "account",
                "add",
                "--name",
                "Primary Gmail",
                "--email",
                "primary@example.com",
                "--login",
                "primary@example.com",
                "--password",
                "app-password",
                "--imap-host",
                "imap.example.com",
                "--imap-port",
                "993",
                "--imap-security",
                "tls",
                "--smtp-host",
                "smtp.example.com",
                "--smtp-port",
                "587",
                "--smtp-security",
                "start_tls",
                "--format",
                "json",
            ],
            &repository,
            &secret_store,
            &tester,
        )
        .expect("新增账户应成功");

        let output = run_with_args_and_dependencies(
            ["account", "list", "--format", "json"],
            &repository,
            &secret_store,
            &tester,
        )
        .expect("列出账户应成功");
        let parsed = serde_json::from_str::<Value>(&output).expect("输出必须是合法 JSON");
        let metadata = fs::read_to_string(&store_path).expect("元数据文件应可读");

        assert_eq!(parsed.as_array().map(|items| items.len()), Some(1));
        assert_eq!(parsed[0]["credential_state"], "stored");
        assert!(
            !metadata.contains("app-password"),
            "JSON 元数据文件不应包含密码"
        );
    }

    #[test]
    fn rejects_account_add_without_password_flag() {
        let error = run_with_args_and_test_store(
            [
                "account",
                "add",
                "--name",
                "Primary Gmail",
                "--email",
                "primary@example.com",
                "--login",
                "primary@example.com",
                "--imap-host",
                "imap.example.com",
                "--imap-port",
                "993",
                "--imap-security",
                "tls",
                "--smtp-host",
                "smtp.example.com",
                "--smtp-port",
                "587",
                "--smtp-security",
                "start_tls",
            ],
            unique_store_path(),
        )
        .expect_err("缺少密码参数时必须报错");

        assert_eq!(
            error,
            AppError::InvalidCliArgs {
                message: "缺少参数: --password".to_string(),
            }
        );
    }

    #[test]
    fn reports_failed_live_probe_for_unreachable_ports() {
        let unreachable_imap_port = reserve_unused_port();
        let smtp = TcpListener::bind("127.0.0.1:0").expect("应能绑定 SMTP 测试端口");

        let output = run_with_args_and_test_store(
            [
                "account",
                "test",
                "--name",
                "Primary Gmail",
                "--email",
                "primary@example.com",
                "--login",
                "primary@example.com",
                "--imap-host",
                "127.0.0.1",
                "--imap-port",
                &unreachable_imap_port.to_string(),
                "--imap-security",
                "none",
                "--smtp-host",
                "127.0.0.1",
                "--smtp-port",
                &smtp
                    .local_addr()
                    .expect("应能读取 SMTP 地址")
                    .port()
                    .to_string(),
                "--smtp-security",
                "none",
                "--format",
                "json",
            ],
            unique_store_path(),
        )
        .expect("实时探测命令应返回结构化结果");
        let parsed = serde_json::from_str::<Value>(&output).expect("输出必须是合法 JSON");

        assert_eq!(parsed["status"], "failed");
        assert!(
            parsed["summary"]
                .as_str()
                .is_some_and(|summary| summary.contains("实时探测未通过")),
            "CLI 应返回实时探测失败语义"
        );
    }

    #[test]
    fn reports_live_probe_success_when_servers_are_reachable() {
        let imap = TcpListener::bind("127.0.0.1:0").expect("应能绑定 IMAP 测试端口");
        let smtp = TcpListener::bind("127.0.0.1:0").expect("应能绑定 SMTP 测试端口");

        let output = run_with_args_and_test_store(
            [
                "account",
                "test",
                "--name",
                "Primary Gmail",
                "--email",
                "primary@example.com",
                "--login",
                "primary@example.com",
                "--imap-host",
                "127.0.0.1",
                "--imap-port",
                &imap
                    .local_addr()
                    .expect("应能读取 IMAP 地址")
                    .port()
                    .to_string(),
                "--imap-security",
                "none",
                "--smtp-host",
                "127.0.0.1",
                "--smtp-port",
                &smtp
                    .local_addr()
                    .expect("应能读取 SMTP 地址")
                    .port()
                    .to_string(),
                "--smtp-security",
                "none",
                "--format",
                "json",
            ],
            unique_store_path(),
        )
        .expect("实时探测成功时应返回结构化结果");
        let parsed = serde_json::from_str::<Value>(&output).expect("输出必须是合法 JSON");

        assert_eq!(parsed["status"], "passed");
        assert!(
            parsed["summary"]
                .as_str()
                .is_some_and(|summary| summary.contains("实时探测通过")),
            "CLI 应返回实时探测成功语义"
        );
    }

    #[test]
    fn rejects_unsupported_formats() {
        let error = run_with_args_and_test_store(
            ["workspace", "bootstrap", "--format", "yaml"],
            unique_store_path(),
        )
        .expect_err("不支持的格式必须报错");

        assert_eq!(
            error,
            crate::domain::error::AppError::UnsupportedFormat {
                format: "yaml".to_string(),
            }
        );
    }

    fn unique_store_path() -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("系统时间应晚于 epoch")
            .as_nanos();

        std::env::temp_dir()
            .join("twill-tests")
            .join(format!("cli-accounts-{suffix}.json"))
    }

    fn reserve_unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("应能分配空闲端口");
        let port = listener.local_addr().expect("应能读取本地地址").port();
        drop(listener);
        port
    }

    fn run_with_args_and_test_store<I, S>(
        args: I,
        store_path: std::path::PathBuf,
    ) -> Result<String, AppError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let repository = JsonFileAccountRepository::new(store_path);
        let secret_store = InMemorySecretStore::default();
        let tester = LiveAccountConnectionTester::default();

        run_with_args_and_dependencies(args, &repository, &secret_store, &tester)
    }
}
