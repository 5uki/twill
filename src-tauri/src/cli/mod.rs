use crate::domain::account::{
    AccountConnectionCheckTarget, AccountConnectionStatus, AccountConnectionTestInput,
    AccountConnectionTestResult, AccountCredentialState, AddAccountInput, MailSecurity,
    MailServerConfig,
};
use crate::domain::error::AppError;
use crate::domain::workspace::WorkspaceMailboxKind;
use crate::infra::account_preflight::LiveAccountConnectionTester;
use crate::infra::account_secret_store::KeyringAccountSecretStore;
use crate::infra::account_store::JsonFileAccountRepository;
use crate::infra::workspace_store::JsonFileWorkspaceRepository;
use crate::infra::workspace_sync_source::SeededWorkspaceSyncSource;
use crate::services::account_service::{
    self, AccountConnectionTester, AccountRepository, AccountSecretStore,
};
use crate::services::workspace_service::{
    self, WorkspaceMessageFilter, WorkspaceSnapshotRepository, WorkspaceSyncSource,
};
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
    let workspace_repository = JsonFileWorkspaceRepository::new(default_workspace_store_path());
    let secret_store = KeyringAccountSecretStore::from_default_service_name();
    let tester = LiveAccountConnectionTester::default();
    let sync_source = SeededWorkspaceSyncSource;

    run_with_args_and_dependencies(
        args,
        &repository,
        &workspace_repository,
        &secret_store,
        &tester,
        &sync_source,
    )
}

fn run_with_args_and_dependencies<I, S, R, WorkspaceRepo, SecretStore, Tester, Source>(
    args: I,
    repository: &R,
    workspace_repository: &WorkspaceRepo,
    secret_store: &SecretStore,
    tester: &Tester,
    sync_source: &Source,
) -> Result<String, AppError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
    R: AccountRepository,
    WorkspaceRepo: WorkspaceSnapshotRepository,
    SecretStore: AccountSecretStore,
    Tester: AccountConnectionTester,
    Source: WorkspaceSyncSource,
{
    let args = args
        .into_iter()
        .map(|value| value.as_ref().to_string())
        .collect::<Vec<_>>();

    match args.as_slice() {
        [workspace, bootstrap] if workspace == "workspace" && bootstrap == "bootstrap" => {
            render_workspace_bootstrap(OutputFormat::Text, workspace_repository)
        }
        [workspace, bootstrap, flag, value]
            if workspace == "workspace" && bootstrap == "bootstrap" && flag == "--format" =>
        {
            render_workspace_bootstrap(parse_output_format(value)?, workspace_repository)
        }
        [sync, run] if sync == "sync" && run == "run" => {
            render_sync_run(OutputFormat::Text, repository, workspace_repository, sync_source)
        }
        [sync, run, rest @ ..] if sync == "sync" && run == "run" => {
            let mut flags = parse_flags(rest)?;
            let format = take_output_format(&mut flags)?;

            ensure_no_unknown_flags(&flags)?;
            render_sync_run(format, repository, workspace_repository, sync_source)
        }
        [mailbox, list] if mailbox == "mailbox" && list == "list" => {
            render_mailbox_list(OutputFormat::Text, workspace_repository)
        }
        [mailbox, list, rest @ ..] if mailbox == "mailbox" && list == "list" => {
            let mut flags = parse_flags(rest)?;
            let format = take_output_format(&mut flags)?;

            ensure_no_unknown_flags(&flags)?;
            render_mailbox_list(format, workspace_repository)
        }
        [message, list] if message == "message" && list == "list" => {
            render_message_list(
                OutputFormat::Text,
                workspace_repository,
                WorkspaceMessageFilter::default(),
            )
        }
        [message, list, rest @ ..] if message == "message" && list == "list" => {
            let mut flags = parse_flags(rest)?;
            let format = take_output_format(&mut flags)?;
            let filter = parse_message_filter(&mut flags)?;

            ensure_no_unknown_flags(&flags)?;
            render_message_list(format, workspace_repository, filter)
        }
        [message, read, rest @ ..] if message == "message" && read == "read" => {
            let mut flags = parse_flags(rest)?;
            let format = take_output_format(&mut flags)?;
            let message_id = take_required_flag(&mut flags, "--id")?;

            ensure_no_unknown_flags(&flags)?;
            render_message_read(format, workspace_repository, &message_id)
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
                "  sync run [--format text|json]\n",
                "  mailbox list [--format text|json]\n",
                "  message list [--account <account-id>] [--mailbox <inbox|spam_junk>] ",
                "[--verification-only <true|false>] [--format text|json]\n",
                "  message read --id <message-id> [--format text|json]\n",
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

fn render_workspace_bootstrap<R>(
    format: OutputFormat,
    workspace_repository: &R,
) -> Result<String, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    let snapshot = workspace_service::load_workspace_bootstrap(workspace_repository)?;

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

fn render_sync_run<R, W, S>(
    format: OutputFormat,
    repository: &R,
    workspace_repository: &W,
    sync_source: &S,
) -> Result<String, AppError>
where
    R: AccountRepository,
    W: WorkspaceSnapshotRepository,
    S: WorkspaceSyncSource,
{
    let snapshot =
        workspace_service::sync_workspace(repository, workspace_repository, sync_source)?;
    let message_count = snapshot
        .message_groups
        .iter()
        .flat_map(|group| group.items.iter())
        .count();
    let verification_count = snapshot
        .message_groups
        .iter()
        .flat_map(|group| group.items.iter())
        .filter(|message| message.has_code || message.has_link)
        .count();
    let sync_summary = snapshot
        .sync_status
        .as_ref()
        .map(|status| status.summary.as_str())
        .unwrap_or("收件箱缓存已更新");

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&snapshot).map_err(|error| AppError::Serialization {
                message: error.to_string(),
            })
        }
        OutputFormat::Text => Ok(format!(
            "收件箱同步已完成\n状态: {sync_summary}\n消息数: {message_count}\n验证消息: {verification_count}\n当前选中: {}",
            snapshot.selected_message.subject
        )),
    }
}

fn render_mailbox_list<R>(
    format: OutputFormat,
    workspace_repository: &R,
) -> Result<String, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    let mailboxes = workspace_service::list_workspace_mailboxes(workspace_repository)?;

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&mailboxes).map_err(|error| AppError::Serialization {
                message: error.to_string(),
            })
        }
        OutputFormat::Text => {
            if mailboxes.is_empty() {
                return Ok("当前没有已缓存的邮箱。".to_string());
            }

            let lines = mailboxes
                .iter()
                .map(|mailbox| {
                    format!(
                        "- {} | {} | 总计 {} | 未读 {} | 验证 {}",
                        mailbox.account_name,
                        mailbox.label,
                        mailbox.total_count,
                        mailbox.unread_count,
                        mailbox.verification_count
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            Ok(format!("已缓存邮箱\n{lines}"))
        }
    }
}

fn render_message_list<R>(
    format: OutputFormat,
    workspace_repository: &R,
    filter: WorkspaceMessageFilter,
) -> Result<String, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    let messages = workspace_service::list_workspace_messages(workspace_repository, &filter)?;

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&messages).map_err(|error| AppError::Serialization {
                message: error.to_string(),
            })
        }
        OutputFormat::Text => {
            if messages.is_empty() {
                return Ok("当前筛选条件下没有缓存消息。".to_string());
            }

            let lines = messages
                .iter()
                .map(|message| {
                    format!(
                        "- {} | {} | {} | {}",
                        message.account_name,
                        message.mailbox_label,
                        message.subject,
                        message.received_at
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            Ok(format!("已缓存消息\n{lines}"))
        }
    }
}

fn render_message_read<R>(
    format: OutputFormat,
    workspace_repository: &R,
    message_id: &str,
) -> Result<String, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    let message = workspace_service::read_workspace_message(workspace_repository, message_id)?;

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&message).map_err(|error| AppError::Serialization {
                message: error.to_string(),
            })
        }
        OutputFormat::Text => Ok(format!(
            "缓存消息详情\n主题: {}\n账号: {}\n邮箱: {}\n站点: {}\n摘要: {}\n正文缓存: {}\n同步时间: {}",
            message.subject,
            message.account_name,
            message.mailbox_label,
            message.site_hint,
            message.summary,
            if message.prefetched_body {
                "已预抓取"
            } else {
                "仅元数据"
            },
            message.synced_at
        )),
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

fn parse_message_filter(
    flags: &mut BTreeMap<String, String>,
) -> Result<WorkspaceMessageFilter, AppError> {
    let account_id = flags.remove("--account");
    let mailbox_kind = match flags.remove("--mailbox") {
        Some(value) => Some(parse_mailbox_kind(&value)?),
        None => None,
    };
    let verification_only = match flags.remove("--verification-only") {
        Some(value) => parse_bool_flag(&value, "--verification-only")?,
        None => false,
    };

    Ok(WorkspaceMessageFilter {
        account_id,
        mailbox_kind,
        verification_only,
    })
}

fn parse_mailbox_kind(value: &str) -> Result<WorkspaceMailboxKind, AppError> {
    match value {
        "inbox" => Ok(WorkspaceMailboxKind::Inbox),
        "spam_junk" => Ok(WorkspaceMailboxKind::SpamJunk),
        other => Err(AppError::Validation {
            field: "mailbox".to_string(),
            message: format!("不支持的邮箱类型: {other}"),
        }),
    }
}

fn parse_bool_flag(value: &str, field: &str) -> Result<bool, AppError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(AppError::Validation {
            field: field.to_string(),
            message: format!("布尔参数只支持 true/false，收到 {other}"),
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

fn default_workspace_store_path() -> PathBuf {
    crate::infra::workspace_store::default_workspace_store_path()
}

#[cfg(test)]
mod tests {
    use super::run_with_args_and_dependencies;
    use crate::domain::error::AppError;
    use crate::infra::account_preflight::LiveAccountConnectionTester;
    use crate::infra::account_store::JsonFileAccountRepository;
    use crate::infra::workspace_store::JsonFileWorkspaceRepository;
    use crate::infra::workspace_sync_source::SeededWorkspaceSyncSource;
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
        let workspace_path = unique_workspace_store_path();
        let workspace_repository = JsonFileWorkspaceRepository::new(workspace_path);
        let secret_store = InMemorySecretStore::default();
        let tester = LiveAccountConnectionTester::default();
        let sync_source = SeededWorkspaceSyncSource;

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
            &workspace_repository,
            &secret_store,
            &tester,
            &sync_source,
        )
        .expect("新增账户应成功");

        let output = run_with_args_and_dependencies(
            ["account", "list", "--format", "json"],
            &repository,
            &workspace_repository,
            &secret_store,
            &tester,
            &sync_source,
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
    fn rejects_sync_run_when_no_accounts_exist() {
        let error = run_with_args_and_test_workspace(
            ["sync", "run"],
            unique_store_path(),
            unique_workspace_store_path(),
        )
        .expect_err("没有账户时同步必须报错");

        assert_eq!(
            error,
            AppError::Validation {
                field: "accounts".to_string(),
                message: "请先添加至少一个账户后再同步收件箱".to_string(),
            }
        );
    }

    #[test]
    fn sync_run_persists_snapshot_for_workspace_bootstrap() {
        let account_store_path = unique_store_path();
        let workspace_store_path = unique_workspace_store_path();
        let repository = JsonFileAccountRepository::new(account_store_path.clone());
        let workspace_repository = JsonFileWorkspaceRepository::new(workspace_store_path.clone());
        let secret_store = InMemorySecretStore::default();
        let tester = LiveAccountConnectionTester::default();
        let sync_source = SeededWorkspaceSyncSource;

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
            ],
            &repository,
            &workspace_repository,
            &secret_store,
            &tester,
            &sync_source,
        )
        .expect("新增账户应成功");

        let synced_output = run_with_args_and_dependencies(
            ["sync", "run", "--format", "json"],
            &repository,
            &workspace_repository,
            &secret_store,
            &tester,
            &sync_source,
        )
        .expect("同步命令应成功");
        let synced = serde_json::from_str::<Value>(&synced_output).expect("输出必须是合法 JSON");

        assert_eq!(synced["navigation"][3]["badge"], 1);
        assert_eq!(
            synced["message_groups"][0]["items"][1]["account_name"],
            "Primary Gmail"
        );

        let bootstrap_output = run_with_args_and_dependencies(
            ["workspace", "bootstrap", "--format", "json"],
            &repository,
            &workspace_repository,
            &secret_store,
            &tester,
            &sync_source,
        )
        .expect("读取工作台快照应成功");
        let bootstrap =
            serde_json::from_str::<Value>(&bootstrap_output).expect("输出必须是合法 JSON");

        assert_eq!(
            bootstrap["message_groups"][0]["items"][1]["account_name"],
            "Primary Gmail"
        );
        assert!(
            workspace_store_path.exists(),
            "同步完成后必须把快照持久化到本地缓存"
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

    #[test]
    fn mailbox_list_reads_seed_snapshot_when_workspace_cache_is_empty() {
        let output = run_with_args_and_test_store(
            ["mailbox", "list", "--format", "json"],
            unique_store_path(),
        )
        .expect("读取邮箱列表应成功");
        let parsed = serde_json::from_str::<Value>(&output).expect("输出必须是合法 JSON");
        let mailboxes = parsed.as_array().expect("邮箱列表输出应该是 JSON 数组");

        assert_eq!(mailboxes.len(), 3);
        assert!(mailboxes.iter().any(|mailbox| {
            mailbox["account_id"] == "seed_primary-gmail" && mailbox["label"] == "Inbox"
        }));
        assert!(
            mailboxes
                .iter()
                .any(|mailbox| mailbox["kind"] == "spam_junk")
        );
    }

    #[test]
    fn message_list_filters_synced_cache_by_account_and_mailbox() {
        let repository = JsonFileAccountRepository::new(unique_store_path());
        let workspace_repository = JsonFileWorkspaceRepository::new(unique_workspace_store_path());
        let secret_store = InMemorySecretStore::default();
        let tester = LiveAccountConnectionTester::default();
        let sync_source = SeededWorkspaceSyncSource;

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
            ],
            &repository,
            &workspace_repository,
            &secret_store,
            &tester,
            &sync_source,
        )
        .expect("新增账号应成功");

        run_with_args_and_dependencies(
            ["sync", "run", "--format", "json"],
            &repository,
            &workspace_repository,
            &secret_store,
            &tester,
            &sync_source,
        )
        .expect("同步命令应成功");

        let output = run_with_args_and_dependencies(
            [
                "message",
                "list",
                "--account",
                "acct_primary-example-com",
                "--mailbox",
                "spam_junk",
                "--verification-only",
                "true",
                "--format",
                "json",
            ],
            &repository,
            &workspace_repository,
            &secret_store,
            &tester,
            &sync_source,
        )
        .expect("按账号与邮箱筛选消息应成功");
        let parsed = serde_json::from_str::<Value>(&output).expect("输出必须是合法 JSON");
        let messages = parsed.as_array().expect("消息列表输出应该是 JSON 数组");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["id"], "msg_notion_welcome");
        assert_eq!(messages[0]["mailbox_label"], "Spam/Junk");
        assert_eq!(messages[0]["account_id"], "acct_primary-example-com");
    }

    #[test]
    fn message_read_returns_prefetched_detail_from_seed_snapshot() {
        let output = run_with_args_and_test_store(
            [
                "message",
                "read",
                "--id",
                "msg_linear_verify",
                "--format",
                "json",
            ],
            unique_store_path(),
        )
        .expect("读取缓存消息详情应成功");
        let parsed = serde_json::from_str::<Value>(&output).expect("输出必须是合法 JSON");

        assert_eq!(parsed["id"], "msg_linear_verify");
        assert_eq!(parsed["site_hint"], "linear.app");
        assert_eq!(parsed["verification_link"], "https://linear.app/login");
        assert_eq!(parsed["prefetched_body"], true);
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

    fn unique_workspace_store_path() -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("系统时间应晚于 epoch")
            .as_nanos();

        std::env::temp_dir()
            .join("twill-tests")
            .join(format!("cli-workspace-{suffix}.json"))
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
        let workspace_repository = JsonFileWorkspaceRepository::new(unique_workspace_store_path());
        let secret_store = InMemorySecretStore::default();
        let tester = LiveAccountConnectionTester::default();
        let sync_source = SeededWorkspaceSyncSource;

        run_with_args_and_dependencies(
            args,
            &repository,
            &workspace_repository,
            &secret_store,
            &tester,
            &sync_source,
        )
    }

    fn run_with_args_and_test_workspace<I, S>(
        args: I,
        store_path: std::path::PathBuf,
        workspace_store_path: std::path::PathBuf,
    ) -> Result<String, AppError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let repository = JsonFileAccountRepository::new(store_path);
        let workspace_repository = JsonFileWorkspaceRepository::new(workspace_store_path);
        let secret_store = InMemorySecretStore::default();
        let tester = LiveAccountConnectionTester::default();
        let sync_source = SeededWorkspaceSyncSource;

        run_with_args_and_dependencies(
            args,
            &repository,
            &workspace_repository,
            &secret_store,
            &tester,
            &sync_source,
        )
    }
}
