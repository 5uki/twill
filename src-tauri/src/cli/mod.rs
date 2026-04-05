use crate::domain::account::{
    AccountConnectionCheckTarget, AccountConnectionStatus, AccountConnectionTestInput,
    AccountConnectionTestResult, AccountCredentialState, AddAccountInput, MailSecurity,
    MailServerConfig,
};
use crate::domain::error::AppError;
use crate::domain::workspace::{
    MessageCategory, MessageReadState, MessageStatus, WorkspaceMailboxKind, WorkspaceMessageAction,
};
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
        [message, open, rest @ ..] if message == "message" && open == "open" => {
            let mut flags = parse_flags(rest)?;
            let format = take_output_format(&mut flags)?;
            let message_id = take_required_flag(&mut flags, "--id")?;

            ensure_no_unknown_flags(&flags)?;
            render_message_open(format, workspace_repository, &message_id)
        }
        [message, original, rest @ ..] if message == "message" && original == "original" => {
            let mut flags = parse_flags(rest)?;
            let format = take_output_format(&mut flags)?;
            let message_id = take_required_flag(&mut flags, "--id")?;

            ensure_no_unknown_flags(&flags)?;
            render_message_original(format, workspace_repository, &message_id)
        }
        [message, mark, rest @ ..] if message == "message" && mark == "mark" => {
            let mut flags = parse_flags(rest)?;
            let format = take_output_format(&mut flags)?;
            let message_id = take_required_flag(&mut flags, "--id")?;
            let status = parse_message_status(&take_required_flag(&mut flags, "--status")?)?;

            ensure_no_unknown_flags(&flags)?;
            render_message_mark(format, workspace_repository, &message_id, status)
        }
        [message, read_state, rest @ ..]
            if message == "message" && read_state == "read-state" =>
        {
            let mut flags = parse_flags(rest)?;
            let format = take_output_format(&mut flags)?;
            let message_id = take_required_flag(&mut flags, "--id")?;
            let read_state =
                parse_message_read_state(&take_required_flag(&mut flags, "--state")?)?;

            ensure_no_unknown_flags(&flags)?;
            render_message_read_state(format, workspace_repository, &message_id, read_state)
        }
        [message, action, rest @ ..] if message == "message" && action == "action" => {
            let mut flags = parse_flags(rest)?;
            let format = take_output_format(&mut flags)?;
            let message_id = take_required_flag(&mut flags, "--id")?;
            let action =
                parse_message_action(&take_required_flag(&mut flags, "--action")?)?;

            ensure_no_unknown_flags(&flags)?;
            render_message_action(format, workspace_repository, &message_id, action)
        }
        [site_context, resolve, rest @ ..]
            if site_context == "site-context" && resolve == "resolve" =>
        {
            let mut flags = parse_flags(rest)?;
            let format = take_output_format(&mut flags)?;
            let domain = take_required_flag(&mut flags, "--domain")?;

            ensure_no_unknown_flags(&flags)?;
            render_site_context_resolve(format, workspace_repository, &domain)
        }
        [site_context, confirm, rest @ ..]
            if site_context == "site-context" && confirm == "confirm" =>
        {
            let mut flags = parse_flags(rest)?;
            let format = take_output_format(&mut flags)?;
            let domain = take_required_flag(&mut flags, "--domain")?;
            let label = flags.remove("--label");

            ensure_no_unknown_flags(&flags)?;
            render_site_context_confirm(
                format,
                workspace_repository,
                &domain,
                label.as_deref(),
            )
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
                "鐢ㄦ硶:\n",
                "  workspace bootstrap [--format text|json]\n",
                "  sync run [--format text|json]\n",
                "  mailbox list [--format text|json]\n",
                "  message list [--account <account-id>] [--mailbox <inbox|spam_junk>] ",
                "[--verification-only <true|false>] [--category <registration|security|marketing>] [--site <hostname>] ",
                "[--query <keyword>] [--recent-hours <hours>] [--format text|json]\n",
                "  message read --id <message-id> [--format text|json]\n",
                "  message open --id <message-id> [--format text|json]\n",
                "  message original --id <message-id> [--format text|json]\n",
                "  message mark --id <message-id> --status <pending|processed> [--format text|json]\n",
                "  message read-state --id <message-id> --state <unread|read> [--format text|json]\n",
                "  message action --id <message-id> --action <copy_code|open_link> [--format text|json]\n",
                "  site-context resolve --domain <domain> [--format text|json]\n",
                "  site-context confirm --domain <domain> [--label <label>] [--format text|json]\n",
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
                "Twill workspace bootstrap\n榛樿瑙嗗浘: Recent verification\n鐢熸垚鏃堕棿: {}\n瀵艰埅:\n{}\n褰撳墠閫変腑: {}\n楠岃瘉鐮? {}\n閾炬帴: {}",
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
        .unwrap_or("鏀朵欢绠辩紦瀛樺凡鏇存柊");

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&snapshot).map_err(|error| AppError::Serialization {
                message: error.to_string(),
            })
        }
        OutputFormat::Text => Ok(format!(
            "鏀朵欢绠卞悓姝ュ凡瀹屾垚\n鐘舵€? {sync_summary}\n娑堟伅鏁? {message_count}\n楠岃瘉娑堟伅: {verification_count}\n褰撳墠閫変腑: {}",
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
            "缂撳瓨娑堟伅璇︽儏\n涓婚: {}\n璐﹀彿: {}\n閭: {}\n绔欑偣: {}\n鎽樿: {}\n姝ｆ枃缂撳瓨: {}\n鍚屾鏃堕棿: {}",
            message.subject,
            message.account_name,
            message.mailbox_label,
            message.site_hint,
            message.summary,
            if message.prefetched_body {
                "宸查鎶撳彇"
            } else {
                "浠呭厓鏁版嵁"
            },
            message.synced_at
        )),
    }
}

fn render_message_open<R>(
    format: OutputFormat,
    workspace_repository: &R,
    message_id: &str,
) -> Result<String, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    let result = workspace_service::open_workspace_message(workspace_repository, message_id)?;

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&result).map_err(|error| AppError::Serialization {
                message: error.to_string(),
            })
        }
        OutputFormat::Text => Ok(format!(
            "瀹稿弶澧﹀鈧☉鍫熶紖\nID: {}\n娑撳顣? {}\n瀹歌尪顕伴悩鑸碘偓? {}",
            result.detail.id,
            result.detail.subject,
            if result.detail.read_state == crate::domain::workspace::MessageReadState::Read {
                "read"
            } else {
                "unread"
            }
        )),
    }
}

fn render_message_original<R>(
    format: OutputFormat,
    workspace_repository: &R,
    message_id: &str,
) -> Result<String, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    let result =
        workspace_service::open_workspace_message_original(workspace_repository, message_id)?;

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&result).map_err(|error| AppError::Serialization {
                message: error.to_string(),
            })
        }
        OutputFormat::Text => Ok(format!(
            "瀹稿弶澧﹀鈧崢鐔奉潗闁喕娆nID: {}\n閸樼喎顫愰崗銉ュ經: {}",
            result.message_id, result.original_url
        )),
    }
}

fn render_message_mark<R>(
    format: OutputFormat,
    workspace_repository: &R,
    message_id: &str,
    status: MessageStatus,
) -> Result<String, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    let snapshot = workspace_service::update_workspace_message_status(
        workspace_repository,
        message_id,
        status,
    )?;

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&snapshot).map_err(|error| AppError::Serialization {
                message: error.to_string(),
            })
        }
        OutputFormat::Text => {
            let action = match status {
                MessageStatus::Pending => "已撤销已处理",
                MessageStatus::Processed => "已标记为已处理",
            };

            Ok(format!(
                "娑堟伅鐘舵€佸凡鏇存柊\n娑堟伅 ID: {message_id}\n缁撴灉: {action}\n褰撳墠閫変腑: {}",
                snapshot.selected_message.subject
            ))
        }
    }
}

fn render_message_read_state<R>(
    format: OutputFormat,
    workspace_repository: &R,
    message_id: &str,
    read_state: MessageReadState,
) -> Result<String, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    let snapshot = workspace_service::update_workspace_message_read_state(
        workspace_repository,
        message_id,
        read_state,
    )?;

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&snapshot).map_err(|error| AppError::Serialization {
                message: error.to_string(),
            })
        }
        OutputFormat::Text => {
            let action = match read_state {
                MessageReadState::Unread => "已标记为未读",
                MessageReadState::Read => "已标记为已读",
            };

            Ok(format!(
                "消息已读状态已更新\n消息 ID: {message_id}\n结果: {action}\n当前选中: {}",
                snapshot.selected_message.subject
            ))
        }
    }
}

fn render_message_action<R>(
    format: OutputFormat,
    workspace_repository: &R,
    message_id: &str,
    action: WorkspaceMessageAction,
) -> Result<String, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    let result = workspace_service::apply_workspace_message_action(
        workspace_repository,
        message_id,
        action,
    )?;

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&result).map_err(|error| AppError::Serialization {
                message: error.to_string(),
            })
        }
        OutputFormat::Text => {
            let action_summary = match action {
                WorkspaceMessageAction::CopyCode => format!(
                    "宸插鍒堕獙璇佺爜: {}",
                    result.copied_value.as_deref().unwrap_or("-")
                ),
                WorkspaceMessageAction::OpenLink => format!(
                    "宸叉墦寮€閾炬帴: {}",
                    result.opened_url.as_deref().unwrap_or("-")
                ),
            };

            Ok(format!(
                "娑堟伅鍔ㄤ綔宸插畬鎴怽n娑堟伅 ID: {message_id}\n缁撴灉: {action_summary}\n褰撳墠鐘舵€? processed"
            ))
        }
    }
}

fn render_site_context_resolve<R>(
    format: OutputFormat,
    workspace_repository: &R,
    domain: &str,
) -> Result<String, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    let resolution =
        workspace_service::resolve_workspace_site_context(workspace_repository, domain)?;

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&resolution).map_err(|error| AppError::Serialization {
                message: error.to_string(),
            })
        }
        OutputFormat::Text => {
            if let Some(matched_site) = &resolution.matched_site {
                return Ok(format!(
                    "褰撳墠绔欑偣宸插尮閰峔n杈撳叆: {domain}\n褰掍竴鍖栧煙鍚? {}\n鍛戒腑绔欑偣: {} ({})",
                    resolution.normalized_domain.as_deref().unwrap_or("-"),
                    matched_site.label,
                    matched_site.hostname
                ));
            }

            let candidates = resolution
                .candidate_sites
                .iter()
                .map(|site| format!("- {} ({})", site.label, site.hostname))
                .collect::<Vec<_>>()
                .join("\n");

            if candidates.is_empty() {
                Ok(format!(
                    "当前站点未命中\n输入: {domain}\n归一化域名: {}\n候选站点: 无",
                    resolution.normalized_domain.as_deref().unwrap_or("-")
                ))
            } else {
                Ok(format!(
                    "当前站点未命中\n输入: {domain}\n归一化域名: {}\n候选站点:\n{}",
                    resolution.normalized_domain.as_deref().unwrap_or("-"),
                    candidates
                ))
            }
        }
    }
}

fn render_site_context_confirm<R>(
    format: OutputFormat,
    workspace_repository: &R,
    domain: &str,
    label: Option<&str>,
) -> Result<String, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    let snapshot = workspace_service::confirm_workspace_site(workspace_repository, domain, label)?;
    let normalized_domain =
        workspace_service::resolve_workspace_site_context(workspace_repository, domain)?
            .normalized_domain;

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&snapshot).map_err(|error| AppError::Serialization {
                message: error.to_string(),
            })
        }
        OutputFormat::Text => Ok(format!(
            "已确认站点\n输入: {domain}\n归一化域名: {}\n站点总数: {}",
            normalized_domain.as_deref().unwrap_or("-"),
            snapshot.site_summaries.len()
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
                            "- {} <{}> | 鍑嵁 {} | IMAP {}:{} {:?} | SMTP {}:{} {:?}",
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

                Ok(format!("宸蹭繚瀛樿处鎴穃n{lines}"))
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
            "璐︽埛宸蹭繚瀛榎nID: {}\n鍚嶇О: {}\n閭: {}\n鍑嵁: {}\nIMAP: {}:{} ({:?})\nSMTP: {}:{} ({:?})",
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
        AccountConnectionStatus::Passed => "閫氳繃",
        AccountConnectionStatus::Failed => "澶辫触",
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
                AccountConnectionStatus::Passed => "閫氳繃",
                AccountConnectionStatus::Failed => "澶辫触",
            };

            format!("- {target}: {check_status} | {}", check.message)
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "璐︽埛杩炴帴瀹炴椂鎺㈡祴\n鐘舵€? {status}\n缁撹: {}\n{checks}",
        result.summary
    )
}

fn format_credential_state(state: AccountCredentialState) -> &'static str {
    match state {
        AccountCredentialState::Missing => "缺失",
        AccountCredentialState::Stored => "已保存",
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
    let category = match flags.remove("--category") {
        Some(value) => Some(parse_message_category(&value)?),
        None => None,
    };
    let site_hint = flags.remove("--site");
    let query = flags.remove("--query");
    let recent_hours = match flags.remove("--recent-hours") {
        Some(value) => Some(parse_recent_hours(&value)?),
        None => None,
    };

    Ok(WorkspaceMessageFilter {
        account_id,
        mailbox_kind,
        verification_only,
        category,
        site_hint,
        query,
        recent_hours,
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

fn parse_message_category(value: &str) -> Result<MessageCategory, AppError> {
    match value {
        "registration" => Ok(MessageCategory::Registration),
        "security" => Ok(MessageCategory::Security),
        "marketing" => Ok(MessageCategory::Marketing),
        other => Err(AppError::Validation {
            field: "category".to_string(),
            message: format!("不支持的消息分类: {other}"),
        }),
    }
}

fn parse_message_status(value: &str) -> Result<MessageStatus, AppError> {
    match value {
        "pending" => Ok(MessageStatus::Pending),
        "processed" => Ok(MessageStatus::Processed),
        other => Err(AppError::Validation {
            field: "status".to_string(),
            message: format!("不支持的消息状态: {other}"),
        }),
    }
}

fn parse_message_read_state(value: &str) -> Result<MessageReadState, AppError> {
    match value {
        "unread" => Ok(MessageReadState::Unread),
        "read" => Ok(MessageReadState::Read),
        other => Err(AppError::Validation {
            field: "state".to_string(),
            message: format!("不支持的已读状态: {other}"),
        }),
    }
}

fn parse_message_action(value: &str) -> Result<WorkspaceMessageAction, AppError> {
    match value {
        "copy_code" => Ok(WorkspaceMessageAction::CopyCode),
        "open_link" => Ok(WorkspaceMessageAction::OpenLink),
        other => Err(AppError::Validation {
            field: "action".to_string(),
            message: format!("不支持的消息动作: {other}"),
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

fn parse_recent_hours(value: &str) -> Result<u32, AppError> {
    value.parse::<u32>().map_err(|_| AppError::Validation {
        field: "recent_hours".to_string(),
        message: format!("recent_hours 必须是正整数，收到 {value}"),
    })
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
            "鏂囨湰杈撳嚭鑷冲皯瑕佸寘鍚粯璁ゅ伐浣滃彴瑙嗗浘"
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
        let parsed = serde_json::from_str::<Value>(&output).expect("杈撳嚭蹇呴』鏄悎娉?JSON");
        let metadata = fs::read_to_string(&store_path).expect("鍏冩暟鎹枃浠跺簲鍙");

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
        let smtp = TcpListener::bind("127.0.0.1:0").expect("搴旇兘缁戝畾 SMTP 娴嬭瘯绔彛");

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
                    .expect("搴旇兘璇诲彇 SMTP 鍦板潃")
                    .port()
                    .to_string(),
                "--smtp-security",
                "none",
                "--format",
                "json",
            ],
            unique_store_path(),
        )
        .expect("瀹炴椂鎺㈡祴鍛戒护搴旇繑鍥炵粨鏋勫寲缁撴灉");
        let parsed = serde_json::from_str::<Value>(&output).expect("杈撳嚭蹇呴』鏄悎娉?JSON");

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
        let imap = TcpListener::bind("127.0.0.1:0").expect("搴旇兘缁戝畾 IMAP 娴嬭瘯绔彛");
        let smtp = TcpListener::bind("127.0.0.1:0").expect("搴旇兘缁戝畾 SMTP 娴嬭瘯绔彛");

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
                    .expect("搴旇兘璇诲彇 IMAP 鍦板潃")
                    .port()
                    .to_string(),
                "--imap-security",
                "none",
                "--smtp-host",
                "127.0.0.1",
                "--smtp-port",
                &smtp
                    .local_addr()
                    .expect("搴旇兘璇诲彇 SMTP 鍦板潃")
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
        let parsed = serde_json::from_str::<Value>(&output).expect("杈撳嚭蹇呴』鏄悎娉?JSON");

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
        let synced =
            serde_json::from_str::<Value>(&synced_output).expect("杈撳嚭蹇呴』鏄悎娉?JSON");

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
        .expect("璇诲彇宸ヤ綔鍙板揩鐓у簲鎴愬姛");
        let bootstrap =
            serde_json::from_str::<Value>(&bootstrap_output).expect("杈撳嚭蹇呴』鏄悎娉?JSON");

        assert_eq!(
            bootstrap["message_groups"][0]["items"][1]["account_name"],
            "Primary Gmail"
        );
        assert!(
            workspace_store_path.exists(),
            "鍚屾瀹屾垚鍚庡繀椤绘妸蹇収鎸佷箙鍖栧埌鏈湴缂撳瓨"
        );
    }

    #[test]
    fn rejects_unsupported_formats() {
        let error = run_with_args_and_test_store(
            ["workspace", "bootstrap", "--format", "yaml"],
            unique_store_path(),
        )
        .expect_err("涓嶆敮鎸佺殑鏍煎紡蹇呴』鎶ラ敊");

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
        let parsed = serde_json::from_str::<Value>(&output).expect("杈撳嚭蹇呴』鏄悎娉?JSON");
        let mailboxes = parsed
            .as_array()
            .expect("閭鍒楄〃杈撳嚭搴旇鏄?JSON 鏁扮粍");

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
        .expect("鎸夎处鍙蜂笌閭绛涢€夋秷鎭簲鎴愬姛");
        let parsed = serde_json::from_str::<Value>(&output).expect("杈撳嚭蹇呴』鏄悎娉?JSON");
        let messages = parsed
            .as_array()
            .expect("娑堟伅鍒楄〃杈撳嚭搴旇鏄?JSON 鏁扮粍");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["id"], "msg_notion_welcome");
        assert_eq!(messages[0]["mailbox_label"], "Spam/Junk");
        assert_eq!(messages[0]["account_id"], "acct_primary-example-com");
    }

    #[test]
    fn message_list_supports_category_and_query_filters() {
        let output = run_with_args_and_test_store(
            [
                "message",
                "list",
                "--category",
                "security",
                "--query",
                "362149",
                "--format",
                "json",
            ],
            unique_store_path(),
        )
        .expect("分类和查询筛选应成功");
        let parsed =
            serde_json::from_str::<Value>(&output).expect("鏉堟挸鍤箛鍛淬€忛弰顖氭値濞?JSON");
        let messages = parsed.as_array().expect("消息列表输出应是 JSON 数组");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["id"], "msg_github_security");
    }

    #[test]
    fn message_list_supports_exact_site_filter() {
        let output = run_with_args_and_test_store(
            [
                "message",
                "list",
                "--site",
                "github.com",
                "--format",
                "json",
            ],
            unique_store_path(),
        )
        .expect("閹稿鐝悙鍦翱绾喛绻冨銈嗙Х閹垰绨查幋鎰");
        let parsed = serde_json::from_str::<Value>(&output)
            .expect("閺夊牊鎸搁崵顓＄疀閸涙番鈧繘寮伴姘€ゆ繛?JSON");
        let messages = parsed.as_array().expect("消息列表输出应是 JSON 数组");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["id"], "msg_github_security");
    }

    #[test]
    fn site_context_resolve_returns_exact_match_and_candidates() {
        let exact_output = run_with_args_and_test_store(
            [
                "site-context",
                "resolve",
                "--domain",
                "https://www.github.com/login",
                "--format",
                "json",
            ],
            unique_store_path(),
        )
        .expect("缁旀瑧鍋ｇ划鍓р€橀崠褰掑帳閸涙垝鎶ゆ惔鏃€鍨氶崝?");
        let exact = serde_json::from_str::<Value>(&exact_output)
            .expect("閺夊牊鎸搁崵顓＄疀閸涙番鈧繘寮伴姘€ゆ繛?JSON");
        let candidate_output = run_with_args_and_test_store(
            [
                "site-context",
                "resolve",
                "--domain",
                "lin",
                "--format",
                "json",
            ],
            unique_store_path(),
        )
        .expect("閸婃瑩鈧鐝悙纭呅掗弸鎰嚒娴犮倕绨查幋鎰");
        let candidate = serde_json::from_str::<Value>(&candidate_output)
            .expect("閺夊牊鎸搁崵顓＄疀閸涙番鈧繘寮伴姘€ゆ繛?JSON");

        assert_eq!(exact["normalized_domain"], "github.com");
        assert_eq!(exact["matched_site"]["hostname"], "github.com");
        assert_eq!(candidate["matched_site"], Value::Null);
        assert_eq!(candidate["candidate_sites"][0]["hostname"], "linear.app");
    }

    #[test]
    fn message_list_supports_recent_hours_filter() {
        let repository = JsonFileAccountRepository::new(unique_store_path());
        let workspace_store_path = unique_workspace_store_path();
        let workspace_repository = JsonFileWorkspaceRepository::new(workspace_store_path.clone());
        let secret_store = InMemorySecretStore::default();
        let tester = LiveAccountConnectionTester::default();
        let sync_source = SeededWorkspaceSyncSource;
        let seed_snapshot_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../src/data/workspace-bootstrap.json");
        let mut snapshot = serde_json::from_str::<Value>(
            &fs::read_to_string(&seed_snapshot_path).expect("seed snapshot 鎼存棁顕氶崣顖濐嚢"),
        )
        .expect("seed snapshot 韫囧懘銆忛弰顖氭値濞?JSON");

        snapshot["generated_at"] = Value::String("2026-04-05T09:00:00Z".to_string());
        snapshot["message_groups"][0]["items"][0]["received_at"] =
            Value::String("2026-04-01T08:58:00Z".to_string());
        snapshot["message_details"][0]["received_at"] =
            Value::String("2026-04-01T08:58:00Z".to_string());

        fs::create_dir_all(
            workspace_store_path
                .parent()
                .expect("workspace 璺緞搴斿寘鍚埗鐩綍"),
        )
        .expect("娴嬭瘯鐩綍搴斿彲鍒涘缓");
        fs::write(
            &workspace_store_path,
            serde_json::to_string_pretty(&snapshot).expect("快照 JSON 应可序列化"),
        )
        .expect("娴嬭瘯蹇収搴斿彲鍐欏叆");

        let output = run_with_args_and_dependencies(
            [
                "message",
                "list",
                "--verification-only",
                "true",
                "--recent-hours",
                "48",
                "--format",
                "json",
            ],
            &repository,
            &workspace_repository,
            &secret_store,
            &tester,
            &sync_source,
        )
        .expect("recent-hours 缁涙盯鈧绨查幋鎰");
        let parsed = serde_json::from_str::<Value>(&output)
            .expect("閸涙垝鎶ゆ潏鎾冲毉韫囧懘銆忛弰顖氭値濞?JSON");
        let messages = parsed.as_array().expect("消息列表输出应是 JSON 数组");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["id"], "msg_linear_verify");
    }

    #[test]
    fn message_action_marks_processed_and_removes_matching_extract() {
        let output = run_with_args_and_test_store(
            [
                "message",
                "action",
                "--id",
                "msg_github_security",
                "--action",
                "copy_code",
                "--format",
                "json",
            ],
            unique_store_path(),
        )
        .expect("楂樹环鍊煎姩浣滃懡浠ゅ簲鎴愬姛");
        let parsed =
            serde_json::from_str::<Value>(&output).expect("鍛戒护杈撳嚭蹇呴』鏄悎娉?JSON");

        assert_eq!(parsed["action"], "copy_code");
        assert_eq!(parsed["copied_value"], "362149");
        assert_eq!(
            parsed["snapshot"]["selected_message"]["status"],
            "processed"
        );
        assert!(
            parsed["snapshot"]["extracts"]
                .as_array()
                .is_some_and(|items| items.iter().all(|item| item["id"] != "extract_github_code"))
        );
    }

    #[test]
    fn message_open_marks_it_read_and_returns_updated_snapshot() {
        let output = run_with_args_and_test_store(
            [
                "message",
                "open",
                "--id",
                "msg_github_security",
                "--format",
                "json",
            ],
            unique_store_path(),
        )
        .expect("open message 閸涙垝鎶ゆ惔鏃€鍨氶崝?");
        let parsed = serde_json::from_str::<Value>(&output)
            .expect("閸涙垝鎶ゆ潏鎾冲毉韫囧懘銆忛弰顖氭値濞?JSON");

        assert_eq!(parsed["detail"]["id"], "msg_github_security");
        assert_eq!(parsed["detail"]["read_state"], "read");
        assert_eq!(parsed["snapshot"]["selected_message"]["status"], "pending");
    }

    #[test]
    fn message_original_returns_original_url_and_marks_message_read() {
        let output = run_with_args_and_test_store(
            [
                "message",
                "original",
                "--id",
                "msg_linear_verify",
                "--format",
                "json",
            ],
            unique_store_path(),
        )
        .expect("open original 閸涙垝鎶ゆ惔鏃€鍨氶崝?");
        let parsed = serde_json::from_str::<Value>(&output)
            .expect("閸涙垝鎶ゆ潏鎾冲毉韫囧懘銆忛弰顖氭値濞?JSON");

        assert!(
            parsed["original_url"]
                .as_str()
                .is_some_and(|url| url.contains("msg_linear_verify"))
        );
        assert_eq!(
            parsed["snapshot"]["message_details"][1]["read_state"],
            "read"
        );
    }

    #[test]
    fn site_context_confirm_adds_manual_site_to_snapshot() {
        let output = run_with_args_and_test_store(
            [
                "site-context",
                "confirm",
                "--domain",
                "https://vercel.com/login",
                "--format",
                "json",
            ],
            unique_store_path(),
        )
        .expect("confirm site 閸涙垝鎶ゆ惔鏃€鍨氶崝?");
        let parsed = serde_json::from_str::<Value>(&output)
            .expect("閸涙垝鎶ゆ潏鎾冲毉韫囧懘銆忛弰顖氭値濞?JSON");

        assert!(
            parsed["site_summaries"]
                .as_array()
                .is_some_and(|sites| sites.iter().any(|site| site["hostname"] == "vercel.com"))
        );
    }

    #[test]
    fn message_mark_updates_snapshot_and_persists_status_change() {
        let account_store_path = unique_store_path();
        let workspace_store_path = unique_workspace_store_path();
        let repository = JsonFileAccountRepository::new(account_store_path);
        let workspace_repository = JsonFileWorkspaceRepository::new(workspace_store_path.clone());
        let secret_store = InMemorySecretStore::default();
        let tester = LiveAccountConnectionTester::default();
        let sync_source = SeededWorkspaceSyncSource;

        let output = run_with_args_and_dependencies(
            [
                "message",
                "mark",
                "--id",
                "msg_github_security",
                "--status",
                "processed",
                "--format",
                "json",
            ],
            &repository,
            &workspace_repository,
            &secret_store,
            &tester,
            &sync_source,
        )
        .expect("閺嶅洩顔囧鎻掝槱閻炲棗绨查幋鎰");
        let parsed =
            serde_json::from_str::<Value>(&output).expect("鏉堟挸鍤箛鍛淬€忛弰顖氭値濞?JSON");

        assert_eq!(parsed["selected_message"]["status"], "processed");
        assert_eq!(
            parsed["site_summaries"].as_array().and_then(|sites| {
                sites
                    .iter()
                    .find(|site| site["hostname"] == "github.com")
                    .and_then(|site| site["pending_count"].as_u64())
            }),
            Some(0)
        );

        let persisted_output = run_with_args_and_dependencies(
            [
                "message",
                "read",
                "--id",
                "msg_github_security",
                "--format",
                "json",
            ],
            &repository,
            &workspace_repository,
            &secret_store,
            &tester,
            &sync_source,
        )
        .expect("鎼存棁顕氶懗鍊燁嚢閸掓澘鍑￠弴瀛樻煀閻ㄥ嫮绱︾€涙濮搁幀?");
        let persisted = serde_json::from_str::<Value>(&persisted_output)
            .expect("鏉堟挸鍤箛鍛淬€忛弰顖氭値濞?JSON");

        assert_eq!(persisted["status"], "processed");
        assert!(workspace_store_path.exists());
    }

    #[test]
    fn message_read_state_updates_snapshot_and_persists_read_flag() {
        let account_store_path = unique_store_path();
        let workspace_store_path = unique_workspace_store_path();
        let repository = JsonFileAccountRepository::new(account_store_path);
        let workspace_repository = JsonFileWorkspaceRepository::new(workspace_store_path.clone());
        let secret_store = InMemorySecretStore::default();
        let tester = LiveAccountConnectionTester::default();
        let sync_source = SeededWorkspaceSyncSource;

        let output = run_with_args_and_dependencies(
            [
                "message",
                "read-state",
                "--id",
                "msg_github_security",
                "--state",
                "read",
                "--format",
                "json",
            ],
            &repository,
            &workspace_repository,
            &secret_store,
            &tester,
            &sync_source,
        )
        .expect("更新消息已读状态应成功");
        let parsed = serde_json::from_str::<Value>(&output).expect("消息快照输出应为 JSON");

        assert_eq!(parsed["selected_message"]["read_state"], "read");
        assert_eq!(parsed["selected_message"]["status"], "pending");

        let persisted_output = run_with_args_and_dependencies(
            [
                "message",
                "read",
                "--id",
                "msg_github_security",
                "--format",
                "json",
            ],
            &repository,
            &workspace_repository,
            &secret_store,
            &tester,
            &sync_source,
        )
        .expect("读取持久化消息详情应成功");
        let persisted =
            serde_json::from_str::<Value>(&persisted_output).expect("消息详情输出应为 JSON");

        assert_eq!(persisted["read_state"], "read");
        assert_eq!(persisted["status"], "pending");
        assert!(workspace_store_path.exists());
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
        let parsed = serde_json::from_str::<Value>(&output).expect("杈撳嚭蹇呴』鏄悎娉?JSON");

        assert_eq!(parsed["id"], "msg_linear_verify");
        assert_eq!(parsed["site_hint"], "linear.app");
        assert_eq!(parsed["verification_link"], "https://linear.app/login");
        assert_eq!(parsed["prefetched_body"], true);
    }

    fn unique_store_path() -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("绯荤粺鏃堕棿搴旀櫄浜?epoch")
            .as_nanos();

        std::env::temp_dir()
            .join("twill-tests")
            .join(format!("cli-accounts-{suffix}.json"))
    }

    fn unique_workspace_store_path() -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("绯荤粺鏃堕棿搴旀櫄浜?epoch")
            .as_nanos();

        std::env::temp_dir()
            .join("twill-tests")
            .join(format!("cli-workspace-{suffix}.json"))
    }

    fn reserve_unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("搴旇兘鍒嗛厤绌洪棽绔彛");
        let port = listener
            .local_addr()
            .expect("搴旇兘璇诲彇鏈湴鍦板潃")
            .port();
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
