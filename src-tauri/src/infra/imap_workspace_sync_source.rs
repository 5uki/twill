use crate::domain::account::{AccountSummary, MailSecurity};
use crate::domain::error::AppError;
use crate::domain::workspace::{
    MessageCategory, MessageReadState, MessageStatus, NavigationItem, WorkspaceBootstrapSnapshot,
    WorkspaceExtractItem, WorkspaceExtractKind, WorkspaceMailboxKind, WorkspaceMessageDetail,
    WorkspaceMessageItem, WorkspaceSiteSummary, WorkspaceSyncPhase, WorkspaceSyncState,
    WorkspaceSyncStatus, WorkspaceViewId,
};
use crate::services::account_service::AccountSecretStore;
use crate::services::workspace_service::{
    WorkspaceSyncSource, rebuild_mailboxes, rebuild_message_groups, rebuild_site_summaries,
    update_navigation_badges,
};
use chrono::{DateTime, Duration as ChronoDuration, SecondsFormat, Utc};
use imap::types::Flag;
use mail_parser::MessageParser;
use native_tls::TlsConnector;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedMailboxMessage {
    pub mailbox_kind: WorkspaceMailboxKind,
    pub mailbox_label: String,
    pub remote_id: String,
    pub read_state: MessageReadState,
    pub raw_message: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FetchedAccountMailboxSnapshot {
    pub folders: Vec<String>,
    pub messages: Vec<FetchedMailboxMessage>,
}

pub trait ImapAccountSyncClient {
    fn fetch_account_snapshot(
        &self,
        account: &AccountSummary,
        password: &str,
    ) -> Result<FetchedAccountMailboxSnapshot, AppError>;
}

pub struct LiveImapWorkspaceSyncSource<'a, S, C> {
    secret_store: &'a S,
    client: &'a C,
}

impl<'a, S, C> LiveImapWorkspaceSyncSource<'a, S, C> {
    pub fn new(secret_store: &'a S, client: &'a C) -> Self {
        Self {
            secret_store,
            client,
        }
    }
}

impl<S, C> WorkspaceSyncSource for LiveImapWorkspaceSyncSource<'_, S, C>
where
    S: AccountSecretStore,
    C: ImapAccountSyncClient,
{
    fn build_snapshot(
        &self,
        accounts: &[AccountSummary],
        previous_snapshot: Option<&WorkspaceBootstrapSnapshot>,
    ) -> Result<WorkspaceBootstrapSnapshot, AppError> {
        let synced_at = current_timestamp();
        let previous_statuses = build_previous_status_index(previous_snapshot);
        let previous_selected_id =
            previous_snapshot.map(|snapshot| snapshot.selected_message.id.clone());
        let mut folders = BTreeSet::new();
        let mut details = Vec::new();

        for account in accounts {
            let password = self
                .secret_store
                .read_secret(&account.id)?
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| AppError::Validation {
                    field: "account.credential".to_string(),
                    message: format!(
                        "账号 {} 缺少系统安全存储密码，无法读取真实收件箱",
                        account.display_name
                    ),
                })?;

            let fetched = self.client.fetch_account_snapshot(account, &password)?;
            folders.extend(
                fetched
                    .folders
                    .into_iter()
                    .filter(|folder| !folder.trim().is_empty()),
            );

            for message in fetched.messages {
                details.push(build_message_detail(
                    account,
                    message,
                    &synced_at,
                    &previous_statuses,
                )?);
            }
        }

        details.sort_by(|left, right| right.received_at.cmp(&left.received_at));
        let selected_message =
            choose_selected_message(&details, previous_selected_id.as_deref(), &synced_at);
        let message_groups = rebuild_message_groups(
            details
                .iter()
                .map(build_message_item_from_detail)
                .collect::<Vec<_>>(),
            previous_snapshot
                .map(|snapshot| snapshot.message_groups.as_slice())
                .unwrap_or(&[]),
        );
        let mailboxes = rebuild_mailboxes(&message_groups);
        let site_summaries = build_site_summaries(previous_snapshot, &details, &selected_message);
        let extracts = build_extracts(&details, &synced_at);
        let mut navigation = previous_snapshot
            .map(|snapshot| snapshot.navigation.clone())
            .unwrap_or_else(default_navigation);
        update_navigation_badges(
            &mut navigation,
            &message_groups,
            site_summaries.len(),
            accounts.len(),
        );

        Ok(WorkspaceBootstrapSnapshot {
            app_name: previous_snapshot
                .map(|snapshot| snapshot.app_name.clone())
                .unwrap_or_else(|| "Twill".to_string()),
            generated_at: synced_at.clone(),
            default_view: previous_snapshot
                .map(|snapshot| snapshot.default_view)
                .unwrap_or(WorkspaceViewId::RecentVerification),
            navigation,
            mailboxes,
            message_groups,
            selected_message,
            message_details: details.clone(),
            extracts,
            site_summaries,
            sync_status: Some(build_sync_status(
                accounts.len(),
                details.len(),
                previous_snapshot.is_some(),
                folders.into_iter().collect(),
            )),
        })
    }
}

#[derive(Debug, Clone)]
pub struct LiveImapAccountSyncClient {
    timeout: Duration,
    max_messages_per_mailbox: usize,
}

impl LiveImapAccountSyncClient {
    pub fn new(timeout: Duration, max_messages_per_mailbox: usize) -> Self {
        Self {
            timeout,
            max_messages_per_mailbox,
        }
    }
}

impl Default for LiveImapAccountSyncClient {
    fn default() -> Self {
        Self::new(Duration::from_secs(20), 30)
    }
}

impl ImapAccountSyncClient for LiveImapAccountSyncClient {
    fn fetch_account_snapshot(
        &self,
        account: &AccountSummary,
        password: &str,
    ) -> Result<FetchedAccountMailboxSnapshot, AppError> {
        match account.imap.security {
            MailSecurity::Tls => {
                let tls = build_tls_connector(account)?;
                let client = imap::connect(
                    (account.imap.host.as_str(), account.imap.port),
                    account.imap.host.as_str(),
                    &tls,
                )
                .map_err(|error| imap_connect_error(account, error))?;
                let mut session = client
                    .login(account.login.as_str(), password)
                    .map_err(|error| imap_login_error(account, error.0))?;
                let result = fetch_account_snapshot_from_session(
                    &mut session,
                    self.max_messages_per_mailbox,
                );
                let _ = session.logout();
                result
            }
            MailSecurity::StartTls => {
                let tls = build_tls_connector(account)?;
                let client = imap::connect_starttls(
                    (account.imap.host.as_str(), account.imap.port),
                    account.imap.host.as_str(),
                    &tls,
                )
                .map_err(|error| imap_connect_error(account, error))?;
                let mut session = client
                    .login(account.login.as_str(), password)
                    .map_err(|error| imap_login_error(account, error.0))?;
                let result = fetch_account_snapshot_from_session(
                    &mut session,
                    self.max_messages_per_mailbox,
                );
                let _ = session.logout();
                result
            }
            MailSecurity::None => {
                let stream = connect_plain_imap_stream(
                    account.imap.host.as_str(),
                    account.imap.port,
                    self.timeout,
                )
                .map_err(|error| imap_connect_error(account, error))?;
                let mut client = imap::Client::new(stream);
                client
                    .read_greeting()
                    .map_err(|error| imap_connect_error(account, error))?;
                let mut session = client
                    .login(account.login.as_str(), password)
                    .map_err(|error| imap_login_error(account, error.0))?;
                let result = fetch_account_snapshot_from_session(
                    &mut session,
                    self.max_messages_per_mailbox,
                );
                let _ = session.logout();
                result
            }
        }
    }
}

#[derive(Debug, Clone)]
struct ParsedMessageMetadata {
    identity: String,
    subject: String,
    sender: String,
    received_at: String,
    summary: String,
    body_text: Option<String>,
    extracted_code: Option<String>,
    verification_link: Option<String>,
    site_hint: String,
    category: MessageCategory,
}

#[derive(Debug, Clone)]
struct ResolvedMailbox {
    name: String,
    label: String,
    kind: WorkspaceMailboxKind,
}

fn build_tls_connector(account: &AccountSummary) -> Result<TlsConnector, AppError> {
    TlsConnector::builder()
        .build()
        .map_err(|error| AppError::Validation {
            field: "imap".to_string(),
            message: format!(
                "账号 {} 创建 IMAP TLS 连接失败: {error}",
                account.display_name
            ),
        })
}

fn connect_plain_imap_stream(
    host: &str,
    port: u16,
    timeout: Duration,
) -> std::io::Result<TcpStream> {
    let addresses = (host, port).to_socket_addrs()?;
    let mut last_error = None;

    for address in addresses {
        match TcpStream::connect_timeout(&address, timeout) {
            Ok(stream) => {
                let _ = stream.set_read_timeout(Some(timeout));
                let _ = stream.set_write_timeout(Some(timeout));
                return Ok(stream);
            }
            Err(error) => last_error = Some(error),
        }
    }

    Err(last_error.unwrap_or_else(|| std::io::Error::other("无法连接到 IMAP 主机")))
}

fn fetch_account_snapshot_from_session<T: Read + Write>(
    session: &mut imap::Session<T>,
    max_messages_per_mailbox: usize,
) -> Result<FetchedAccountMailboxSnapshot, AppError> {
    let mut folders = vec!["Inbox".to_string()];
    let mut messages = fetch_mailbox_messages(
        session,
        &ResolvedMailbox {
            name: "INBOX".to_string(),
            label: "Inbox".to_string(),
            kind: WorkspaceMailboxKind::Inbox,
        },
        max_messages_per_mailbox,
    )?;

    if let Some(spam_mailbox) = discover_spam_mailbox(session) {
        folders.push(spam_mailbox.label.clone());
        messages.extend(fetch_mailbox_messages(
            session,
            &spam_mailbox,
            max_messages_per_mailbox,
        )?);
    }

    Ok(FetchedAccountMailboxSnapshot { folders, messages })
}

fn discover_spam_mailbox<T: Read + Write>(
    session: &mut imap::Session<T>,
) -> Option<ResolvedMailbox> {
    if let Ok(names) = session.list(None, Some("*")) {
        for mailbox in names.iter() {
            let name = mailbox.name();
            if looks_like_spam_mailbox(name) {
                return Some(ResolvedMailbox {
                    name: name.to_string(),
                    label: "Spam/Junk".to_string(),
                    kind: WorkspaceMailboxKind::SpamJunk,
                });
            }
        }
    }

    for candidate in [
        "Junk",
        "Junk E-mail",
        "Junk Email",
        "Spam",
        "Bulk Mail",
        "[Gmail]/Spam",
        "垃圾邮件",
    ] {
        if session.select(candidate).is_ok() {
            return Some(ResolvedMailbox {
                name: candidate.to_string(),
                label: "Spam/Junk".to_string(),
                kind: WorkspaceMailboxKind::SpamJunk,
            });
        }
    }

    None
}

fn fetch_mailbox_messages<T: Read + Write>(
    session: &mut imap::Session<T>,
    mailbox: &ResolvedMailbox,
    max_messages_per_mailbox: usize,
) -> Result<Vec<FetchedMailboxMessage>, AppError> {
    let mailbox_info =
        session
            .select(mailbox.name.as_str())
            .map_err(|error| AppError::Validation {
                field: "imap".to_string(),
                message: format!("选择邮箱 {} 失败: {error}", mailbox.name),
            })?;

    if mailbox_info.exists == 0 {
        return Ok(Vec::new());
    }

    let max_messages_per_mailbox = u32::try_from(max_messages_per_mailbox).unwrap_or(u32::MAX);
    let start = if mailbox_info.exists > max_messages_per_mailbox {
        mailbox_info.exists - max_messages_per_mailbox + 1
    } else {
        1
    };
    let sequence_set = format!("{start}:{}", mailbox_info.exists);
    let fetches = session
        .fetch(sequence_set, "(FLAGS RFC822)")
        .map_err(|error| AppError::Validation {
            field: "imap".to_string(),
            message: format!("拉取邮箱 {} 的邮件失败: {error}", mailbox.name),
        })?;

    Ok(fetches
        .iter()
        .filter_map(|fetch| {
            let body = fetch.body()?;
            Some(FetchedMailboxMessage {
                mailbox_kind: mailbox.kind,
                mailbox_label: mailbox.label.clone(),
                remote_id: String::new(),
                read_state: if fetch.flags().contains(&Flag::Seen) {
                    MessageReadState::Read
                } else {
                    MessageReadState::Unread
                },
                raw_message: body.to_vec(),
            })
        })
        .collect())
}

fn build_message_detail(
    account: &AccountSummary,
    message: FetchedMailboxMessage,
    synced_at: &str,
    previous_statuses: &BTreeMap<String, MessageStatus>,
) -> Result<WorkspaceMessageDetail, AppError> {
    let metadata = parse_message_metadata(&message, synced_at)?;
    let id = build_message_id(
        account.id.as_str(),
        message.mailbox_kind,
        metadata.identity.as_str(),
    );
    let status = previous_statuses
        .get(&id)
        .copied()
        .unwrap_or(MessageStatus::Pending);
    let body_text = metadata.body_text.clone();

    Ok(WorkspaceMessageDetail {
        id,
        account_id: account.id.clone(),
        subject: metadata.subject,
        sender: metadata.sender,
        account_name: account.display_name.clone(),
        mailbox_id: format!(
            "{}/{}",
            account.id,
            mailbox_storage_key(message.mailbox_kind)
        ),
        mailbox_label: message.mailbox_label,
        received_at: metadata.received_at,
        category: metadata.category,
        status,
        read_state: message.read_state,
        site_hint: metadata.site_hint,
        summary: metadata.summary,
        extracted_code: metadata.extracted_code,
        verification_link: metadata.verification_link,
        original_message_url: None,
        body_text: body_text.clone(),
        prefetched_body: body_text.is_some(),
        synced_at: synced_at.to_string(),
    })
}

fn parse_message_metadata(
    message: &FetchedMailboxMessage,
    synced_at: &str,
) -> Result<ParsedMessageMetadata, AppError> {
    let parsed = MessageParser::default()
        .parse(message.raw_message.as_slice())
        .ok_or_else(|| AppError::Serialization {
            message: "解析 IMAP 邮件正文失败".to_string(),
        })?;
    let subject = parsed
        .subject()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("无主题")
        .to_string();
    let sender = parsed
        .return_address()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown@unknown")
        .to_string();
    let received_at = parsed
        .date()
        .map(|date| date.to_rfc3339())
        .and_then(|value| normalize_timestamp(value.as_str()))
        .unwrap_or_else(|| synced_at.to_string());
    let body_text = parsed
        .body_text(0)
        .map(|body| normalize_body_text(body.as_ref()))
        .filter(|value| !value.is_empty());
    let preview = parsed
        .body_preview(160)
        .map(|preview| compact_whitespace(preview.as_ref()))
        .filter(|value| !value.is_empty())
        .or_else(|| {
            body_text
                .as_ref()
                .map(|body| compact_whitespace(body.as_str()))
        })
        .unwrap_or_else(|| subject.clone());
    let links = extract_urls(body_text.as_deref().unwrap_or_default());
    let extracted_code = extract_code(body_text.as_deref().unwrap_or_default());
    let category = classify_message(
        subject.as_str(),
        body_text.as_deref().unwrap_or_default(),
        extracted_code.is_some(),
        !links.is_empty(),
    );
    let verification_link = if category == MessageCategory::Marketing {
        None
    } else {
        links.first().cloned()
    };
    let site_hint = verification_link
        .as_deref()
        .and_then(extract_url_host)
        .or_else(|| sender.split('@').nth(1).map(str::to_string))
        .map(|host| registrable_domain(host.as_str()))
        .unwrap_or_else(|| "unknown".to_string());
    let identity = parsed
        .message_id()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            (!message.remote_id.trim().is_empty()).then_some(message.remote_id.trim().to_string())
        })
        .unwrap_or_else(|| {
            stable_hash_key(&format!(
                "{}|{}|{}|{}",
                subject, sender, received_at, preview
            ))
        });

    Ok(ParsedMessageMetadata {
        identity,
        subject,
        sender,
        received_at,
        summary: truncate_text(preview.as_str(), 120),
        body_text,
        extracted_code,
        verification_link,
        site_hint,
        category,
    })
}

fn build_previous_status_index(
    snapshot: Option<&WorkspaceBootstrapSnapshot>,
) -> BTreeMap<String, MessageStatus> {
    let mut statuses = BTreeMap::new();

    if let Some(snapshot) = snapshot {
        for detail in &snapshot.message_details {
            statuses.insert(detail.id.clone(), detail.status);
        }
        statuses.insert(
            snapshot.selected_message.id.clone(),
            snapshot.selected_message.status,
        );
    }

    statuses
}

fn choose_selected_message(
    details: &[WorkspaceMessageDetail],
    previous_selected_id: Option<&str>,
    synced_at: &str,
) -> WorkspaceMessageDetail {
    if let Some(previous_selected_id) = previous_selected_id
        && let Some(detail) = details
            .iter()
            .find(|detail| detail.id == previous_selected_id)
    {
        return detail.clone();
    }

    if let Some(detail) = details
        .iter()
        .find(|detail| detail.status == MessageStatus::Pending)
        .or_else(|| details.first())
    {
        return detail.clone();
    }

    WorkspaceMessageDetail {
        id: "message_empty".to_string(),
        account_id: String::new(),
        subject: "暂无邮件".to_string(),
        sender: String::new(),
        account_name: String::new(),
        mailbox_id: String::new(),
        mailbox_label: "Inbox".to_string(),
        received_at: synced_at.to_string(),
        category: MessageCategory::Marketing,
        status: MessageStatus::Processed,
        read_state: MessageReadState::Read,
        site_hint: String::new(),
        summary: "当前同步范围内没有拉取到邮件。".to_string(),
        extracted_code: None,
        verification_link: None,
        original_message_url: None,
        body_text: None,
        prefetched_body: false,
        synced_at: synced_at.to_string(),
    }
}

fn build_message_item_from_detail(detail: &WorkspaceMessageDetail) -> WorkspaceMessageItem {
    WorkspaceMessageItem {
        id: detail.id.clone(),
        account_id: detail.account_id.clone(),
        subject: detail.subject.clone(),
        sender: detail.sender.clone(),
        account_name: detail.account_name.clone(),
        mailbox_id: detail.mailbox_id.clone(),
        mailbox_label: detail.mailbox_label.clone(),
        received_at: detail.received_at.clone(),
        category: detail.category,
        status: detail.status,
        read_state: detail.read_state,
        has_code: detail.extracted_code.is_some(),
        has_link: detail.verification_link.is_some(),
        preview: detail.summary.clone(),
        prefetched_body: detail.prefetched_body,
        synced_at: detail.synced_at.clone(),
    }
}

fn build_site_summaries(
    previous_snapshot: Option<&WorkspaceBootstrapSnapshot>,
    details: &[WorkspaceMessageDetail],
    selected_message: &WorkspaceMessageDetail,
) -> Vec<WorkspaceSiteSummary> {
    if details.is_empty() {
        return previous_snapshot
            .map(|snapshot| {
                snapshot
                    .site_summaries
                    .iter()
                    .cloned()
                    .map(|mut summary| {
                        summary.pending_count = 0;
                        summary
                    })
                    .collect()
            })
            .unwrap_or_default();
    }

    let mut existing_summaries = previous_snapshot
        .map(|snapshot| snapshot.site_summaries.clone())
        .unwrap_or_default();
    let mut existing_hosts = existing_summaries
        .iter()
        .map(|summary| summary.hostname.clone())
        .collect::<BTreeSet<_>>();

    for detail in details {
        if detail.site_hint.is_empty() || existing_hosts.contains(detail.site_hint.as_str()) {
            continue;
        }

        existing_summaries.push(WorkspaceSiteSummary {
            id: format!("site_{}", detail.site_hint.replace(['.', '@'], "_")),
            label: display_site_label(detail.site_hint.as_str()),
            hostname: detail.site_hint.clone(),
            pending_count: 0,
            latest_sender: detail.sender.clone(),
        });
        existing_hosts.insert(detail.site_hint.clone());
    }

    rebuild_site_summaries(&existing_summaries, details, selected_message)
}

fn build_extracts(
    details: &[WorkspaceMessageDetail],
    synced_at: &str,
) -> Vec<WorkspaceExtractItem> {
    details
        .iter()
        .filter(|detail| detail.status == MessageStatus::Pending)
        .filter_map(|detail| {
            if let Some(code) = &detail.extracted_code {
                let (progress_percent, expires_label) =
                    build_extract_timing(detail.received_at.as_str(), synced_at, 10);
                return Some(WorkspaceExtractItem {
                    id: format!("extract_{}_code", detail.id),
                    sender: display_site_label(detail.site_hint.as_str()),
                    kind: WorkspaceExtractKind::Code,
                    value: code.clone(),
                    label: String::new(),
                    progress_percent,
                    expires_label,
                });
            }

            detail.verification_link.as_ref().map(|link| {
                let (progress_percent, expires_label) =
                    build_extract_timing(detail.received_at.as_str(), synced_at, 20);
                WorkspaceExtractItem {
                    id: format!("extract_{}_link", detail.id),
                    sender: display_site_label(detail.site_hint.as_str()),
                    kind: WorkspaceExtractKind::Link,
                    value: link.clone(),
                    label: "打开验证链接".to_string(),
                    progress_percent,
                    expires_label,
                }
            })
        })
        .collect()
}

fn build_sync_status(
    account_count: usize,
    message_count: usize,
    has_previous_snapshot: bool,
    folders: Vec<String>,
) -> WorkspaceSyncStatus {
    let phase = if has_previous_snapshot {
        WorkspaceSyncPhase::Incremental
    } else {
        WorkspaceSyncPhase::First
    };
    let summary = if message_count == 0 {
        format!("已同步 {account_count} 个账号，当前同步范围内没有最近邮件")
    } else if phase == WorkspaceSyncPhase::First {
        format!("首次同步完成，已同步 {account_count} 个账号，共 {message_count} 封邮件")
    } else {
        format!("已刷新 {account_count} 个账号，共 {message_count} 封邮件")
    };

    WorkspaceSyncStatus {
        state: WorkspaceSyncState::Ready,
        summary,
        phase: Some(phase),
        poll_interval_minutes: Some(3),
        retention_days: Some(30),
        next_poll_at: Some(
            (Utc::now() + ChronoDuration::minutes(3)).to_rfc3339_opts(SecondsFormat::Secs, true),
        ),
        folders,
    }
}

fn current_timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn default_navigation() -> Vec<NavigationItem> {
    vec![
        NavigationItem {
            id: WorkspaceViewId::RecentVerification,
            label: "Recent verification".to_string(),
            badge: 0,
        },
        NavigationItem {
            id: WorkspaceViewId::AllInbox,
            label: "All inbox".to_string(),
            badge: 0,
        },
        NavigationItem {
            id: WorkspaceViewId::SiteList,
            label: "Sites".to_string(),
            badge: 0,
        },
        NavigationItem {
            id: WorkspaceViewId::Accounts,
            label: "Accounts".to_string(),
            badge: 0,
        },
    ]
}

fn build_message_id(
    account_id: &str,
    mailbox_kind: WorkspaceMailboxKind,
    identity: &str,
) -> String {
    format!(
        "msg_{}",
        stable_hash_key(&format!(
            "{}|{}|{}",
            account_id,
            mailbox_storage_key(mailbox_kind),
            identity
        ))
    )
}

fn stable_hash_key(input: &str) -> String {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn mailbox_storage_key(kind: WorkspaceMailboxKind) -> &'static str {
    match kind {
        WorkspaceMailboxKind::Inbox => "inbox",
        WorkspaceMailboxKind::SpamJunk => "spam-junk",
    }
}

fn looks_like_spam_mailbox(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("spam") || lower.contains("junk")
}

fn normalize_timestamp(value: &str) -> Option<String> {
    DateTime::parse_from_rfc3339(value).ok().map(|timestamp| {
        timestamp
            .with_timezone(&Utc)
            .to_rfc3339_opts(SecondsFormat::Secs, true)
    })
}

fn normalize_body_text(body: &str) -> String {
    body.replace("\r\n", "\n").trim().to_string()
}

fn compact_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_code(body: &str) -> Option<String> {
    let mut current = String::new();

    for character in body.chars() {
        if character.is_ascii_digit() {
            current.push(character);
            continue;
        }

        if (4..=8).contains(&current.len()) {
            return Some(current);
        }

        current.clear();
    }

    (4..=8).contains(&current.len()).then_some(current)
}

fn extract_urls(body: &str) -> Vec<String> {
    body.split_whitespace()
        .filter_map(normalize_url_token)
        .collect()
}

fn normalize_url_token(token: &str) -> Option<String> {
    let trimmed = token.trim_matches(|character: char| {
        matches!(
            character,
            '"' | '\''
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '<'
                | '>'
                | ','
                | '.'
                | ';'
                | '!'
                | '?'
        )
    });

    (trimmed.starts_with("https://") || trimmed.starts_with("http://"))
        .then_some(trimmed.to_string())
}

fn extract_url_host(url: &str) -> Option<String> {
    let without_scheme = url.split_once("://").map(|(_, value)| value).unwrap_or(url);
    let authority = without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default();
    let hostname = authority
        .rsplit('@')
        .next()
        .unwrap_or(authority)
        .strip_prefix("www.")
        .unwrap_or_else(|| authority.rsplit('@').next().unwrap_or(authority))
        .split(':')
        .next()
        .unwrap_or_default()
        .trim_matches('.');

    (!hostname.is_empty()).then_some(hostname.to_string())
}

fn registrable_domain(hostname: &str) -> String {
    let hostname = hostname.trim().trim_matches('.').to_lowercase();
    let labels = hostname.split('.').collect::<Vec<_>>();

    if labels.len() <= 2 {
        return hostname;
    }

    let last = labels[labels.len() - 1];
    let second_last = labels[labels.len() - 2];
    if last.len() == 2 && second_last.len() <= 3 && labels.len() >= 3 {
        return labels[labels.len() - 3..].join(".");
    }

    labels[labels.len() - 2..].join(".")
}

fn classify_message(subject: &str, body: &str, has_code: bool, has_links: bool) -> MessageCategory {
    let normalized = format!("{} {}", subject.to_lowercase(), body.to_lowercase());
    let security_keywords = [
        "验证码",
        "verification code",
        "security code",
        "one-time",
        "one time",
        "otp",
        "2fa",
        "two-factor",
        "authentication code",
        "login code",
        "登录验证码",
        "安全验证码",
        "passcode",
    ];
    let registration_keywords = [
        "verify your email",
        "verify email",
        "confirm your email",
        "activate",
        "activation",
        "complete sign up",
        "finish signing up",
        "magic link",
        "login link",
        "sign in link",
        "验证邮箱",
        "确认邮箱",
        "验证链接",
        "完成注册",
    ];

    if has_code
        || security_keywords
            .iter()
            .any(|keyword| normalized.contains(keyword))
    {
        return MessageCategory::Security;
    }

    if has_links
        && registration_keywords
            .iter()
            .any(|keyword| normalized.contains(keyword))
    {
        return MessageCategory::Registration;
    }

    MessageCategory::Marketing
}

fn truncate_text(value: &str, max_chars: usize) -> String {
    let mut truncated = value.chars().take(max_chars).collect::<String>();

    if value.chars().count() > max_chars {
        truncated.push('…');
    }

    truncated
}

fn build_extract_timing(received_at: &str, synced_at: &str, ttl_minutes: i64) -> (u8, String) {
    let remaining_minutes = DateTime::parse_from_rfc3339(received_at)
        .ok()
        .zip(DateTime::parse_from_rfc3339(synced_at).ok())
        .map(|(received_at, synced_at)| {
            let age = synced_at - received_at;
            (ttl_minutes - age.num_minutes()).max(0)
        })
        .unwrap_or(ttl_minutes);
    let progress = ((remaining_minutes as f32 / ttl_minutes as f32) * 100.0).round();

    (
        progress.clamp(0.0, 100.0) as u8,
        format!("{remaining_minutes}m"),
    )
}

fn display_site_label(hostname: &str) -> String {
    let hostname = registrable_domain(hostname);
    let label = hostname
        .split('.')
        .next()
        .unwrap_or(hostname.as_str())
        .replace('-', " ");

    let mut characters = label.chars();
    let Some(first) = characters.next() else {
        return hostname;
    };

    format!("{}{}", first.to_uppercase(), characters.as_str())
}

fn imap_connect_error(account: &AccountSummary, error: impl std::fmt::Display) -> AppError {
    AppError::Validation {
        field: "imap".to_string(),
        message: format!("账号 {} 连接 IMAP 失败: {error}", account.display_name),
    }
}

fn imap_login_error(account: &AccountSummary, error: impl std::fmt::Display) -> AppError {
    AppError::Validation {
        field: "account.credential".to_string(),
        message: format!(
            "账号 {} 的 IMAP 登录失败，请检查邮箱密码或授权码: {error}",
            account.display_name
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FetchedAccountMailboxSnapshot, FetchedMailboxMessage, ImapAccountSyncClient,
        LiveImapWorkspaceSyncSource,
    };
    use crate::domain::account::{
        AccountCredentialState, AccountSummary, MailSecurity, MailServerConfig,
    };
    use crate::domain::error::AppError;
    use crate::domain::workspace::{MessageReadState, WorkspaceMailboxKind, WorkspaceViewId};
    use crate::services::account_service::AccountSecretStore;
    use crate::services::workspace_service::WorkspaceSyncSource;
    use std::cell::{Cell, RefCell};
    use std::collections::BTreeMap;

    #[derive(Default)]
    struct InMemorySecretStore {
        secrets: RefCell<BTreeMap<String, String>>,
    }

    impl AccountSecretStore for InMemorySecretStore {
        fn save_secret(&self, account_id: &str, secret: &str) -> Result<(), AppError> {
            self.secrets
                .borrow_mut()
                .insert(account_id.to_string(), secret.to_string());
            Ok(())
        }

        fn read_secret(&self, account_id: &str) -> Result<Option<String>, AppError> {
            Ok(self.secrets.borrow().get(account_id).cloned())
        }

        fn delete_secret(&self, account_id: &str) -> Result<(), AppError> {
            self.secrets.borrow_mut().remove(account_id);
            Ok(())
        }

        fn has_secret(&self, account_id: &str) -> Result<bool, AppError> {
            Ok(self.secrets.borrow().contains_key(account_id))
        }
    }

    #[derive(Default)]
    struct FakeImapAccountSyncClient {
        snapshots: RefCell<BTreeMap<String, FetchedAccountMailboxSnapshot>>,
        fetch_count: Cell<u32>,
    }

    impl ImapAccountSyncClient for FakeImapAccountSyncClient {
        fn fetch_account_snapshot(
            &self,
            account: &AccountSummary,
            _password: &str,
        ) -> Result<FetchedAccountMailboxSnapshot, AppError> {
            self.fetch_count.set(self.fetch_count.get() + 1);
            self.snapshots
                .borrow()
                .get(&account.id)
                .cloned()
                .ok_or_else(|| AppError::Storage {
                    message: format!("缺少账号 {} 的测试快照", account.id),
                })
        }
    }

    #[test]
    fn rejects_sync_when_account_secret_is_missing() {
        let secret_store = InMemorySecretStore::default();
        let client = FakeImapAccountSyncClient::default();
        let source = LiveImapWorkspaceSyncSource::new(&secret_store, &client);

        let error = source
            .build_snapshot(&[sample_account()], None)
            .expect_err("缺少系统密码时必须拒绝真实同步");

        assert_eq!(
            error,
            AppError::Validation {
                field: "account.credential".to_string(),
                message: "账号 Primary Gmail 缺少系统安全存储密码，无法读取真实收件箱".to_string(),
            }
        );
        assert_eq!(client.fetch_count.get(), 0);
    }

    #[test]
    fn builds_workspace_snapshot_from_live_imap_messages() {
        let secret_store = InMemorySecretStore::default();
        secret_store
            .save_secret("acct_primary-example-com", "app-password")
            .expect("测试密码应可写入");
        let client = FakeImapAccountSyncClient::default();
        client.snapshots.borrow_mut().insert(
            "acct_primary-example-com".to_string(),
            FetchedAccountMailboxSnapshot {
                folders: vec!["Inbox".to_string(), "Spam/Junk".to_string()],
                messages: vec![
                    FetchedMailboxMessage {
                        mailbox_kind: WorkspaceMailboxKind::Inbox,
                        mailbox_label: "Inbox".to_string(),
                        remote_id: "<github-security@example.com>".to_string(),
                        read_state: MessageReadState::Unread,
                        raw_message: sample_github_security_message(),
                    },
                    FetchedMailboxMessage {
                        mailbox_kind: WorkspaceMailboxKind::Inbox,
                        mailbox_label: "Inbox".to_string(),
                        remote_id: "<linear-verify@example.com>".to_string(),
                        read_state: MessageReadState::Unread,
                        raw_message: sample_linear_registration_message(),
                    },
                    FetchedMailboxMessage {
                        mailbox_kind: WorkspaceMailboxKind::SpamJunk,
                        mailbox_label: "Spam/Junk".to_string(),
                        remote_id: "<notion-news@example.com>".to_string(),
                        read_state: MessageReadState::Read,
                        raw_message: sample_notion_marketing_message(),
                    },
                ],
            },
        );
        let source = LiveImapWorkspaceSyncSource::new(&secret_store, &client);

        let snapshot = source
            .build_snapshot(&[sample_account()], None)
            .expect("真实 IMAP 消息应能映射为工作台快照");

        assert_eq!(snapshot.default_view, WorkspaceViewId::RecentVerification);
        assert_eq!(snapshot.mailboxes.len(), 2);
        assert_eq!(
            snapshot
                .sync_status
                .as_ref()
                .map(|status| status.folders.clone())
                .unwrap_or_default(),
            vec!["Inbox".to_string(), "Spam/Junk".to_string()]
        );
        assert!(snapshot.message_details.iter().any(|detail| detail.subject
            == "GitHub 安全验证码"
            && detail.extracted_code.as_deref() == Some("362149")
            && detail.verification_link.as_deref() == Some("https://github.com/login/device")
            && detail.site_hint == "github.com"));
        assert!(
            snapshot
                .message_details
                .iter()
                .any(|detail| detail.subject == "Verify your Linear email"
                    && detail.category == crate::domain::workspace::MessageCategory::Registration
                    && detail.verification_link.as_deref() == Some("https://linear.app/login"))
        );
        assert!(
            snapshot
                .message_details
                .iter()
                .any(|detail| detail.subject == "Notion weekly roundup"
                    && detail.category == crate::domain::workspace::MessageCategory::Marketing
                    && detail.verification_link.is_none())
        );
        assert_eq!(snapshot.selected_message.subject, "GitHub 安全验证码");
    }

    fn sample_account() -> AccountSummary {
        AccountSummary {
            id: "acct_primary-example-com".to_string(),
            display_name: "Primary Gmail".to_string(),
            email: "primary@example.com".to_string(),
            login: "primary@example.com".to_string(),
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

    fn sample_github_security_message() -> Vec<u8> {
        concat!(
            "From: GitHub <noreply@github.com>\r\n",
            "Date: Sat, 05 Apr 2026 08:58:00 +0000\r\n",
            "Subject: GitHub 安全验证码\r\n",
            "Message-ID: <github-security@example.com>\r\n",
            "Content-Type: text/plain; charset=utf-8\r\n",
            "\r\n",
            "你的 GitHub 登录验证码是 362149。\r\n",
            "也可以点击 https://github.com/login/device 完成验证。\r\n",
        )
        .as_bytes()
        .to_vec()
    }

    fn sample_linear_registration_message() -> Vec<u8> {
        concat!(
            "From: Linear <hello@linear.app>\r\n",
            "Date: Sat, 05 Apr 2026 08:41:00 +0000\r\n",
            "Subject: Verify your Linear email\r\n",
            "Message-ID: <linear-verify@example.com>\r\n",
            "Content-Type: text/plain; charset=utf-8\r\n",
            "\r\n",
            "Click https://linear.app/login to verify your email.\r\n",
        )
        .as_bytes()
        .to_vec()
    }

    fn sample_notion_marketing_message() -> Vec<u8> {
        concat!(
            "From: Notion <team@makenotion.com>\r\n",
            "Date: Thu, 03 Apr 2026 07:12:00 +0000\r\n",
            "Subject: Notion weekly roundup\r\n",
            "Message-ID: <notion-news@example.com>\r\n",
            "Content-Type: text/plain; charset=utf-8\r\n",
            "\r\n",
            "Read more product updates at https://www.notion.so/blog.\r\n",
        )
        .as_bytes()
        .to_vec()
    }
}
