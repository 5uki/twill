use crate::domain::account::AccountSummary;
use crate::domain::error::AppError;
use crate::domain::workspace::{
    MessageCategory, MessageReadState, MessageStatus, NavigationItem, WorkspaceBootstrapSnapshot,
    WorkspaceMailboxKind, WorkspaceMailboxSummary, WorkspaceMessageAction,
    WorkspaceMessageActionResult, WorkspaceMessageDetail, WorkspaceMessageGroup,
    WorkspaceMessageItem, WorkspaceMessageOpenResult, WorkspaceMessageOriginalOpenResult,
    WorkspaceSiteContextResolution, WorkspaceSiteSummary, WorkspaceViewId,
};
use crate::infra::static_workspace;
use crate::services::account_service::AccountRepository;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

pub trait WorkspaceSnapshotRepository {
    fn load_snapshot(&self) -> Result<Option<WorkspaceBootstrapSnapshot>, AppError>;
    fn save_snapshot(&self, snapshot: &WorkspaceBootstrapSnapshot) -> Result<(), AppError>;
}

pub trait WorkspaceSyncSource {
    fn build_snapshot(
        &self,
        accounts: &[AccountSummary],
        previous_snapshot: Option<&WorkspaceBootstrapSnapshot>,
    ) -> Result<WorkspaceBootstrapSnapshot, AppError>;
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct WorkspaceMessageFilter {
    pub account_id: Option<String>,
    pub mailbox_kind: Option<WorkspaceMailboxKind>,
    pub verification_only: bool,
    pub category: Option<MessageCategory>,
    pub site_hint: Option<String>,
    pub query: Option<String>,
    pub recent_hours: Option<u32>,
}

pub fn load_workspace_bootstrap<R>(repository: &R) -> Result<WorkspaceBootstrapSnapshot, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    match repository.load_snapshot() {
        Ok(Some(snapshot)) => Ok(snapshot),
        Ok(None) => Ok(static_workspace::load_snapshot()),
        Err(AppError::Storage { .. }) => Ok(static_workspace::load_snapshot()),
        Err(error) => Err(error),
    }
}

pub fn sync_workspace<A, R, S>(
    account_repository: &A,
    workspace_repository: &R,
    sync_source: &S,
) -> Result<WorkspaceBootstrapSnapshot, AppError>
where
    A: AccountRepository,
    R: WorkspaceSnapshotRepository,
    S: WorkspaceSyncSource,
{
    let accounts = account_repository.list_accounts()?;
    let previous_snapshot = match workspace_repository.load_snapshot() {
        Ok(snapshot) => snapshot,
        Err(AppError::Storage { .. }) => None,
        Err(error) => return Err(error),
    };

    if accounts.is_empty() {
        return Err(AppError::Validation {
            field: "accounts".to_string(),
            message: "请先添加至少一个账户后再同步收件箱".to_string(),
        });
    }

    let snapshot = sync_source.build_snapshot(&accounts, previous_snapshot.as_ref())?;
    workspace_repository.save_snapshot(&snapshot)?;

    Ok(snapshot)
}

pub fn list_workspace_mailboxes<R>(repository: &R) -> Result<Vec<WorkspaceMailboxSummary>, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    Ok(load_workspace_bootstrap(repository)?.mailboxes)
}

pub fn list_workspace_messages<R>(
    repository: &R,
    filter: &WorkspaceMessageFilter,
) -> Result<Vec<WorkspaceMessageItem>, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    let snapshot = load_workspace_bootstrap(repository)?;
    let generated_at = snapshot.generated_at.clone();
    let message_site_hints =
        build_message_site_hint_index(&snapshot.message_details, &snapshot.selected_message);
    let normalized_site_hint = filter.site_hint.as_deref().and_then(normalize_site_input);
    let items = snapshot
        .message_groups
        .into_iter()
        .flat_map(|group| group.items.into_iter())
        .filter(|item| {
            if let Some(account_id) = &filter.account_id
                && &item.account_id != account_id
            {
                return false;
            }

            if let Some(mailbox_kind) = filter.mailbox_kind
                && parse_mailbox_kind(&item.mailbox_label) != Some(mailbox_kind)
            {
                return false;
            }

            if filter.verification_only && !(item.has_code || item.has_link) {
                return false;
            }

            if let Some(category) = filter.category
                && item.category != category
            {
                return false;
            }

            if let Some(site_hint) = normalized_site_hint.as_deref()
                && message_site_hints.get(item.id.as_str()).map(String::as_str) != Some(site_hint)
            {
                return false;
            }

            if !matches_recent_window(
                item.received_at.as_str(),
                generated_at.as_str(),
                filter.recent_hours,
            ) {
                return false;
            }

            matches_message_query(item, filter.query.as_deref())
        })
        .collect::<Vec<_>>();

    Ok(items)
}

pub fn read_workspace_message<R>(
    repository: &R,
    message_id: &str,
) -> Result<WorkspaceMessageDetail, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    let snapshot = load_workspace_bootstrap(repository)?;

    snapshot
        .message_details
        .into_iter()
        .find(|item| item.id == message_id)
        .or_else(|| {
            (snapshot.selected_message.id == message_id).then_some(snapshot.selected_message)
        })
        .ok_or_else(|| AppError::Validation {
            field: "message.id".to_string(),
            message: format!("未找到消息 {message_id}"),
        })
}

pub fn resolve_workspace_site_context<R>(
    repository: &R,
    input: &str,
) -> Result<WorkspaceSiteContextResolution, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    let snapshot = load_workspace_bootstrap(repository)?;

    Ok(resolve_workspace_site_context_from_snapshot(
        &snapshot, input,
    ))
}

pub fn confirm_workspace_site<R>(
    repository: &R,
    input: &str,
    label: Option<&str>,
) -> Result<WorkspaceBootstrapSnapshot, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    let mut snapshot = load_workspace_bootstrap(repository)?;
    let normalized_domain = normalize_site_input(input).ok_or_else(|| AppError::Validation {
        field: "domain".to_string(),
        message: "请输入可识别的站点域名".to_string(),
    })?;
    if !is_confirmable_site_domain(&normalized_domain) {
        return Err(AppError::Validation {
            field: "domain".to_string(),
            message: format!("请输入完整的站点域名，当前值为 {normalized_domain}"),
        });
    }

    let display_label = label
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(normalized_domain.as_str())
        .to_string();

    if let Some(site) = snapshot
        .site_summaries
        .iter_mut()
        .find(|site| site.hostname.eq_ignore_ascii_case(&normalized_domain))
    {
        site.label = display_label;
    } else {
        snapshot.site_summaries.push(WorkspaceSiteSummary {
            id: format!("site_{}", normalized_domain.replace(['.', '@'], "_")),
            label: display_label,
            hostname: normalized_domain,
            pending_count: 0,
            latest_sender: String::new(),
        });
    }

    update_navigation_badges(
        &mut snapshot.navigation,
        &snapshot.message_groups,
        snapshot.site_summaries.len(),
        count_accounts(&snapshot.message_groups),
    );

    repository.save_snapshot(&snapshot)?;

    Ok(snapshot)
}

pub fn open_workspace_message<R>(
    repository: &R,
    message_id: &str,
) -> Result<WorkspaceMessageOpenResult, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    let mut snapshot = load_workspace_bootstrap(repository)?;
    apply_message_read_state_to_snapshot(&mut snapshot, message_id, MessageReadState::Read)?;
    let detail = read_message_detail_from_snapshot(&snapshot, message_id)?;
    repository.save_snapshot(&snapshot)?;

    Ok(WorkspaceMessageOpenResult { detail, snapshot })
}

pub fn open_workspace_message_original<R>(
    repository: &R,
    message_id: &str,
) -> Result<WorkspaceMessageOriginalOpenResult, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    let mut snapshot = load_workspace_bootstrap(repository)?;
    let original_url = read_message_detail_from_snapshot(&snapshot, message_id)?
        .original_message_url
        .ok_or_else(|| AppError::Validation {
            field: "message.original".to_string(),
            message: format!("消息 {message_id} 没有可打开的原始邮件入口"),
        })?;
    apply_message_read_state_to_snapshot(&mut snapshot, message_id, MessageReadState::Read)?;
    repository.save_snapshot(&snapshot)?;

    Ok(WorkspaceMessageOriginalOpenResult {
        message_id: message_id.to_string(),
        original_url,
        snapshot,
    })
}

pub fn apply_workspace_message_action<R>(
    repository: &R,
    message_id: &str,
    action: WorkspaceMessageAction,
) -> Result<WorkspaceMessageActionResult, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    let mut snapshot = load_workspace_bootstrap(repository)?;
    let detail = read_message_detail_from_snapshot(&snapshot, message_id)?;

    let (copied_value, opened_url, extract_kind, extract_value) = match action {
        WorkspaceMessageAction::CopyCode => {
            let copied_value =
                detail
                    .extracted_code
                    .clone()
                    .ok_or_else(|| AppError::Validation {
                        field: "message.action".to_string(),
                        message: format!(
                            "濞戝牊浼?{message_id} 濞屸剝婀侀崣顖氼槻閸掑墎娈戞宀冪槈閻?"
                        ),
                    })?;

            (
                Some(copied_value.clone()),
                None,
                crate::domain::workspace::WorkspaceExtractKind::Code,
                copied_value,
            )
        }
        WorkspaceMessageAction::OpenLink => {
            let opened_url =
                detail
                    .verification_link
                    .clone()
                    .ok_or_else(|| AppError::Validation {
                        field: "message.action".to_string(),
                        message: format!(
                            "濞戝牊浼?{message_id} 濞屸剝婀侀崣顖涘ⅵ瀵偓閻ㄥ嫰鐛欑拠渚€鎽奸幒?"
                        ),
                    })?;

            (
                None,
                Some(opened_url.clone()),
                crate::domain::workspace::WorkspaceExtractKind::Link,
                opened_url,
            )
        }
    };

    apply_message_status_to_snapshot(&mut snapshot, message_id, MessageStatus::Processed)?;
    snapshot
        .extracts
        .retain(|extract| !(extract.kind == extract_kind && extract.value == extract_value));
    repository.save_snapshot(&snapshot)?;

    Ok(WorkspaceMessageActionResult {
        action,
        message_id: message_id.to_string(),
        copied_value,
        opened_url,
        snapshot,
    })
}

pub fn update_workspace_message_status<R>(
    repository: &R,
    message_id: &str,
    status: MessageStatus,
) -> Result<WorkspaceBootstrapSnapshot, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    let mut snapshot = load_workspace_bootstrap(repository)?;
    apply_message_status_to_snapshot(&mut snapshot, message_id, status)?;
    repository.save_snapshot(&snapshot)?;

    Ok(snapshot)
}

pub fn update_workspace_message_read_state<R>(
    repository: &R,
    message_id: &str,
    read_state: MessageReadState,
) -> Result<WorkspaceBootstrapSnapshot, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    let mut snapshot = load_workspace_bootstrap(repository)?;
    apply_message_read_state_to_snapshot(&mut snapshot, message_id, read_state)?;
    repository.save_snapshot(&snapshot)?;

    Ok(snapshot)
}

fn read_message_detail_from_snapshot(
    snapshot: &WorkspaceBootstrapSnapshot,
    message_id: &str,
) -> Result<WorkspaceMessageDetail, AppError> {
    snapshot
        .message_details
        .iter()
        .find(|item| item.id == message_id)
        .cloned()
        .or_else(|| {
            (snapshot.selected_message.id == message_id)
                .then_some(snapshot.selected_message.clone())
        })
        .ok_or_else(|| AppError::Validation {
            field: "message.id".to_string(),
            message: format!("未找到消息 {message_id}"),
        })
}

fn apply_message_status_to_snapshot(
    snapshot: &mut WorkspaceBootstrapSnapshot,
    message_id: &str,
    status: MessageStatus,
) -> Result<(), AppError> {
    let mut items = snapshot
        .message_groups
        .iter()
        .flat_map(|group| group.items.iter().cloned())
        .collect::<Vec<_>>();
    let mut found = false;

    for item in &mut items {
        if item.id == message_id {
            item.status = status;
            if status == MessageStatus::Processed {
                item.read_state = MessageReadState::Read;
            }
            found = true;
        }
    }

    for detail in &mut snapshot.message_details {
        if detail.id == message_id {
            detail.status = status;
            if status == MessageStatus::Processed {
                detail.read_state = MessageReadState::Read;
            }
            found = true;
        }
    }

    if snapshot.selected_message.id == message_id {
        snapshot.selected_message.status = status;
        if status == MessageStatus::Processed {
            snapshot.selected_message.read_state = MessageReadState::Read;
        }
        found = true;
    }

    if !found {
        return Err(AppError::Validation {
            field: "message.id".to_string(),
            message: format!("未找到消息: {message_id}"),
        });
    }

    snapshot.message_groups = rebuild_message_groups(items, &snapshot.message_groups);
    snapshot.mailboxes = rebuild_mailboxes(&snapshot.message_groups);
    snapshot.site_summaries = rebuild_site_summaries(
        &snapshot.site_summaries,
        &snapshot.message_details,
        &snapshot.selected_message,
    );
    update_navigation_badges(
        &mut snapshot.navigation,
        &snapshot.message_groups,
        snapshot.site_summaries.len(),
        count_accounts(&snapshot.message_groups),
    );

    Ok(())
}

fn apply_message_read_state_to_snapshot(
    snapshot: &mut WorkspaceBootstrapSnapshot,
    message_id: &str,
    read_state: MessageReadState,
) -> Result<(), AppError> {
    let mut items = snapshot
        .message_groups
        .iter()
        .flat_map(|group| group.items.iter().cloned())
        .collect::<Vec<_>>();
    let mut found = false;

    for item in &mut items {
        if item.id == message_id {
            item.read_state = read_state;
            found = true;
        }
    }

    for detail in &mut snapshot.message_details {
        if detail.id == message_id {
            detail.read_state = read_state;
            found = true;
        }
    }

    if snapshot.selected_message.id == message_id {
        snapshot.selected_message.read_state = read_state;
        found = true;
    }

    if !found {
        return Err(AppError::Validation {
            field: "message.id".to_string(),
            message: format!("未找到消息 {message_id}"),
        });
    }

    snapshot.message_groups = rebuild_message_groups(items, &snapshot.message_groups);
    snapshot.mailboxes = rebuild_mailboxes(&snapshot.message_groups);
    snapshot.site_summaries = rebuild_site_summaries(
        &snapshot.site_summaries,
        &snapshot.message_details,
        &snapshot.selected_message,
    );
    update_navigation_badges(
        &mut snapshot.navigation,
        &snapshot.message_groups,
        snapshot.site_summaries.len(),
        count_accounts(&snapshot.message_groups),
    );

    Ok(())
}

fn parse_mailbox_kind(label: &str) -> Option<WorkspaceMailboxKind> {
    match label {
        "Inbox" => Some(WorkspaceMailboxKind::Inbox),
        "Spam/Junk" => Some(WorkspaceMailboxKind::SpamJunk),
        _ => None,
    }
}

fn matches_message_query(item: &WorkspaceMessageItem, query: Option<&str>) -> bool {
    let Some(query) = query.map(str::trim).filter(|value| !value.is_empty()) else {
        return true;
    };

    let query = query.to_lowercase();
    let haystacks = [
        item.subject.as_str(),
        item.sender.as_str(),
        item.preview.as_str(),
        item.account_name.as_str(),
        item.mailbox_label.as_str(),
    ];

    haystacks
        .into_iter()
        .any(|value| value.to_lowercase().contains(&query))
}

fn matches_recent_window(received_at: &str, generated_at: &str, recent_hours: Option<u32>) -> bool {
    let Some(recent_hours) = recent_hours else {
        return true;
    };
    let Some(received_seconds) = parse_iso8601_seconds(received_at) else {
        return false;
    };
    let Some(generated_seconds) = parse_iso8601_seconds(generated_at) else {
        return false;
    };
    let window_seconds = i64::from(recent_hours) * 60 * 60;

    received_seconds >= generated_seconds.saturating_sub(window_seconds)
}

fn parse_iso8601_seconds(value: &str) -> Option<i64> {
    if let Ok(timestamp) = value.parse::<i64>() {
        return Some(timestamp);
    }

    let year = value.get(0..4)?.parse::<i32>().ok()?;
    let month = value.get(5..7)?.parse::<u32>().ok()?;
    let day = value.get(8..10)?.parse::<u32>().ok()?;
    let hour = value.get(11..13)?.parse::<u32>().ok()?;
    let minute = value.get(14..16)?.parse::<u32>().ok()?;
    let second = value.get(17..19)?.parse::<u32>().ok()?;

    if value.as_bytes().get(19).copied() != Some(b'Z') && !value.ends_with('Z') {
        return None;
    }

    let days = days_from_civil(year, month, day)?;

    Some(days * 86_400 + i64::from(hour) * 3_600 + i64::from(minute) * 60 + i64::from(second))
}

fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i64> {
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let year = i64::from(year) - i64::from((month <= 2) as u8);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month = i64::from(month);
    let day = i64::from(day);
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;

    Some(era * 146_097 + doe - 719_468)
}

fn build_message_site_hint_index(
    message_details: &[WorkspaceMessageDetail],
    selected_message: &WorkspaceMessageDetail,
) -> BTreeMap<String, String> {
    all_message_details(message_details, selected_message)
        .into_iter()
        .filter_map(|detail| {
            normalize_site_input(&detail.site_hint).map(|site_hint| (detail.id.clone(), site_hint))
        })
        .collect()
}

fn resolve_workspace_site_context_from_snapshot(
    snapshot: &WorkspaceBootstrapSnapshot,
    input: &str,
) -> WorkspaceSiteContextResolution {
    let normalized_domain = normalize_site_input(input);
    let Some(normalized_domain_value) = normalized_domain.clone() else {
        return WorkspaceSiteContextResolution {
            input: input.to_string(),
            normalized_domain: None,
            matched_site: None,
            candidate_sites: Vec::new(),
        };
    };

    let matched_site = snapshot
        .site_summaries
        .iter()
        .find(|site| site.hostname.eq_ignore_ascii_case(&normalized_domain_value))
        .cloned();
    let mut candidate_sites = if matched_site.is_some() {
        Vec::new()
    } else {
        snapshot
            .site_summaries
            .iter()
            .filter(|site| site_matches_candidate(site, &normalized_domain_value))
            .cloned()
            .collect::<Vec<_>>()
    };

    candidate_sites.sort_by(|left, right| {
        site_candidate_score(right, &normalized_domain_value)
            .cmp(&site_candidate_score(left, &normalized_domain_value))
            .then(right.pending_count.cmp(&left.pending_count))
            .then(left.hostname.cmp(&right.hostname))
    });

    WorkspaceSiteContextResolution {
        input: input.to_string(),
        normalized_domain: Some(normalized_domain_value),
        matched_site,
        candidate_sites,
    }
}

fn normalize_site_input(input: &str) -> Option<String> {
    let trimmed = input.trim().to_lowercase();

    if trimmed.is_empty() {
        return None;
    }

    let without_scheme = trimmed
        .split_once("://")
        .map(|(_, value)| value)
        .unwrap_or(trimmed.as_str());
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

fn is_confirmable_site_domain(hostname: &str) -> bool {
    hostname.contains('.')
}

fn site_matches_candidate(site: &WorkspaceSiteSummary, query: &str) -> bool {
    let hostname = site.hostname.to_lowercase();
    let label = site.label.to_lowercase();

    hostname.contains(query) || label.contains(query) || query.contains(&hostname)
}

fn site_candidate_score(site: &WorkspaceSiteSummary, query: &str) -> u8 {
    let hostname = site.hostname.to_lowercase();
    let label = site.label.to_lowercase();

    if hostname.starts_with(query) {
        return 3;
    }

    if label.starts_with(query) {
        return 2;
    }

    1
}

fn rebuild_message_groups(
    items: Vec<WorkspaceMessageItem>,
    existing_groups: &[WorkspaceMessageGroup],
) -> Vec<WorkspaceMessageGroup> {
    let group_labels = existing_groups
        .iter()
        .map(|group| (group.id.as_str(), group.label.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut pending_items = Vec::new();
    let mut processed_items = Vec::new();

    for item in items {
        match item.status {
            MessageStatus::Pending => pending_items.push(item),
            MessageStatus::Processed => processed_items.push(item),
        }
    }

    [
        (
            "pending",
            group_labels
                .get("pending")
                .cloned()
                .unwrap_or_else(|| "待处理".to_string()),
            pending_items,
        ),
        (
            "processed",
            group_labels
                .get("processed")
                .cloned()
                .unwrap_or_else(|| "已处理".to_string()),
            processed_items,
        ),
    ]
    .into_iter()
    .filter_map(|(id, label, items)| {
        (!items.is_empty()).then_some(WorkspaceMessageGroup {
            id: id.to_string(),
            label,
            items,
        })
    })
    .collect()
}

fn rebuild_mailboxes(message_groups: &[WorkspaceMessageGroup]) -> Vec<WorkspaceMailboxSummary> {
    let mut mailboxes = BTreeMap::<(String, WorkspaceMailboxKind), WorkspaceMailboxSummary>::new();

    for item in message_groups.iter().flat_map(|group| group.items.iter()) {
        let kind = parse_mailbox_kind(&item.mailbox_label).unwrap_or(WorkspaceMailboxKind::Inbox);
        let mailbox = mailboxes
            .entry((item.account_id.clone(), kind))
            .or_insert_with(|| WorkspaceMailboxSummary {
                id: item.mailbox_id.clone(),
                account_id: item.account_id.clone(),
                account_name: item.account_name.clone(),
                label: item.mailbox_label.clone(),
                kind,
                total_count: 0,
                unread_count: 0,
                verification_count: 0,
            });

        mailbox.total_count += 1;
        mailbox.unread_count += (item.read_state == MessageReadState::Unread) as u32;
        mailbox.verification_count += (item.has_code || item.has_link) as u32;
    }

    mailboxes.into_values().collect()
}

fn rebuild_site_summaries(
    existing_summaries: &[WorkspaceSiteSummary],
    message_details: &[WorkspaceMessageDetail],
    selected_message: &WorkspaceMessageDetail,
) -> Vec<WorkspaceSiteSummary> {
    #[derive(Default)]
    struct SiteMetrics {
        pending_count: u32,
        latest_sender: String,
        latest_received_at: String,
    }

    let mut metrics = BTreeMap::<String, SiteMetrics>::new();

    for detail in all_message_details(message_details, selected_message) {
        let site_metrics = metrics.entry(detail.site_hint.clone()).or_default();

        if detail.status == MessageStatus::Pending {
            site_metrics.pending_count += 1;
        }

        if detail.received_at >= site_metrics.latest_received_at {
            site_metrics.latest_received_at = detail.received_at.clone();
            site_metrics.latest_sender = detail.sender.clone();
        }
    }

    let mut summaries = existing_summaries
        .iter()
        .map(|summary| {
            let mut next_summary = summary.clone();

            if let Some(site_metrics) = metrics.remove(&summary.hostname) {
                next_summary.pending_count = site_metrics.pending_count;
                next_summary.latest_sender = site_metrics.latest_sender;
            } else {
                next_summary.pending_count = 0;
            }

            next_summary
        })
        .collect::<Vec<_>>();

    summaries.extend(
        metrics
            .into_iter()
            .map(|(hostname, site_metrics)| WorkspaceSiteSummary {
                id: format!("site_{}", hostname.replace(['.', '@'], "_")),
                label: hostname.clone(),
                hostname,
                pending_count: site_metrics.pending_count,
                latest_sender: site_metrics.latest_sender,
            }),
    );

    summaries
}

fn all_message_details<'a>(
    message_details: &'a [WorkspaceMessageDetail],
    selected_message: &'a WorkspaceMessageDetail,
) -> Vec<&'a WorkspaceMessageDetail> {
    let mut details = message_details.iter().collect::<Vec<_>>();

    if !message_details
        .iter()
        .any(|detail| detail.id == selected_message.id)
    {
        details.push(selected_message);
    }

    details
}

fn update_navigation_badges(
    navigation: &mut [NavigationItem],
    message_groups: &[WorkspaceMessageGroup],
    site_count: usize,
    account_count: usize,
) {
    let all_messages = message_groups
        .iter()
        .flat_map(|group| group.items.iter())
        .collect::<Vec<_>>();
    let verification_count = all_messages
        .iter()
        .filter(|message| message.has_code || message.has_link)
        .count();

    for item in navigation {
        item.badge = match item.id {
            WorkspaceViewId::RecentVerification => verification_count as u32,
            WorkspaceViewId::AllInbox => all_messages.len() as u32,
            WorkspaceViewId::SiteList => site_count as u32,
            WorkspaceViewId::Accounts => account_count as u32,
        };
    }
}

fn count_accounts(message_groups: &[WorkspaceMessageGroup]) -> usize {
    message_groups
        .iter()
        .flat_map(|group| group.items.iter().map(|item| item.account_id.as_str()))
        .collect::<BTreeSet<_>>()
        .len()
}

#[cfg(test)]
mod tests {
    use super::{
        WorkspaceMessageFilter, WorkspaceSnapshotRepository, WorkspaceSyncSource,
        apply_workspace_message_action, confirm_workspace_site, list_workspace_mailboxes,
        list_workspace_messages, load_workspace_bootstrap, open_workspace_message,
        open_workspace_message_original, read_workspace_message, resolve_workspace_site_context,
        sync_workspace, update_workspace_message_read_state, update_workspace_message_status,
    };
    use crate::domain::account::{
        AccountCredentialState, AccountSummary, MailSecurity, MailServerConfig,
    };
    use crate::domain::error::AppError;
    use crate::domain::workspace::{
        MessageCategory, MessageReadState, MessageStatus, NavigationItem,
        WorkspaceBootstrapSnapshot, WorkspaceExtractItem, WorkspaceExtractKind,
        WorkspaceMailboxKind, WorkspaceMailboxSummary, WorkspaceMessageAction,
        WorkspaceMessageDetail, WorkspaceMessageGroup, WorkspaceMessageItem, WorkspaceSiteSummary,
        WorkspaceSyncPhase, WorkspaceSyncState, WorkspaceSyncStatus, WorkspaceViewId,
    };
    use crate::services::account_service::AccountRepository;
    use std::cell::RefCell;

    struct InMemoryAccountRepository {
        accounts: RefCell<Vec<AccountSummary>>,
    }

    impl Default for InMemoryAccountRepository {
        fn default() -> Self {
            Self {
                accounts: RefCell::new(Vec::new()),
            }
        }
    }

    impl AccountRepository for InMemoryAccountRepository {
        fn list_accounts(&self) -> Result<Vec<AccountSummary>, AppError> {
            Ok(self.accounts.borrow().clone())
        }

        fn save_account(&self, account: &AccountSummary) -> Result<(), AppError> {
            self.accounts.borrow_mut().push(account.clone());
            Ok(())
        }

        fn delete_account(&self, account_id: &str) -> Result<(), AppError> {
            self.accounts
                .borrow_mut()
                .retain(|account| account.id != account_id);
            Ok(())
        }
    }

    struct InMemoryWorkspaceRepository {
        snapshot: RefCell<Option<WorkspaceBootstrapSnapshot>>,
    }

    impl Default for InMemoryWorkspaceRepository {
        fn default() -> Self {
            Self {
                snapshot: RefCell::new(Some(sample_processing_snapshot("Workspace"))),
            }
        }
    }

    impl WorkspaceSnapshotRepository for InMemoryWorkspaceRepository {
        fn load_snapshot(&self) -> Result<Option<WorkspaceBootstrapSnapshot>, AppError> {
            Ok(self.snapshot.borrow().clone())
        }

        fn save_snapshot(&self, snapshot: &WorkspaceBootstrapSnapshot) -> Result<(), AppError> {
            self.snapshot.borrow_mut().replace(snapshot.clone());
            Ok(())
        }
    }

    struct FixedWorkspaceSyncSource {
        snapshot: WorkspaceBootstrapSnapshot,
    }

    impl WorkspaceSyncSource for FixedWorkspaceSyncSource {
        fn build_snapshot(
            &self,
            _accounts: &[AccountSummary],
            _previous_snapshot: Option<&WorkspaceBootstrapSnapshot>,
        ) -> Result<WorkspaceBootstrapSnapshot, AppError> {
            Ok(self.snapshot.clone())
        }
    }

    #[test]
    fn loads_recent_verification_as_default_view_when_cache_is_empty() {
        let repository = InMemoryWorkspaceRepository {
            snapshot: RefCell::new(None),
        };

        let snapshot = load_workspace_bootstrap(&repository).expect("读取工作台快照应成功");

        assert_eq!(snapshot.default_view, WorkspaceViewId::RecentVerification);
    }

    #[test]
    fn rejects_sync_when_no_accounts_have_been_saved() {
        let accounts = InMemoryAccountRepository::default();
        let workspace = InMemoryWorkspaceRepository {
            snapshot: RefCell::new(None),
        };
        let source = FixedWorkspaceSyncSource {
            snapshot: sample_processing_snapshot("Synced"),
        };

        let error =
            sync_workspace(&accounts, &workspace, &source).expect_err("没有账户时必须拒绝同步");

        assert_eq!(
            error,
            AppError::Validation {
                field: "accounts".to_string(),
                message: "请先添加至少一个账户后再同步收件箱".to_string(),
            }
        );
    }

    #[test]
    fn saves_synced_snapshot_into_workspace_cache() {
        let accounts = InMemoryAccountRepository {
            accounts: RefCell::new(vec![sample_account("Primary Gmail")]),
        };
        let workspace = InMemoryWorkspaceRepository {
            snapshot: RefCell::new(None),
        };
        let source = FixedWorkspaceSyncSource {
            snapshot: sample_processing_snapshot("Synced"),
        };

        let snapshot = sync_workspace(&accounts, &workspace, &source).expect("同步应成功");
        let cached = workspace
            .load_snapshot()
            .expect("读取缓存应成功")
            .expect("同步后必须存在缓存");

        assert_eq!(snapshot.app_name, "Synced");
        assert_eq!(cached.app_name, "Synced");
    }

    #[test]
    fn lists_mailboxes_from_cached_snapshot() {
        let repository = InMemoryWorkspaceRepository::default();

        let mailboxes = list_workspace_mailboxes(&repository).expect("读取邮箱列表应成功");

        assert_eq!(mailboxes.len(), 2);
        assert_eq!(mailboxes[0].kind, WorkspaceMailboxKind::Inbox);
    }

    #[test]
    fn filters_messages_by_category_query_and_site() {
        let repository = InMemoryWorkspaceRepository::default();
        let filter = WorkspaceMessageFilter {
            category: Some(MessageCategory::Security),
            query: Some("362149".to_string()),
            site_hint: Some("github.com".to_string()),
            ..WorkspaceMessageFilter::default()
        };

        let messages = list_workspace_messages(&repository, &filter).expect("读取消息列表应成功");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, "msg_github_security");
    }

    #[test]
    fn filters_verification_messages_by_recent_hours_window() {
        let mut snapshot = sample_processing_snapshot("48h window");
        snapshot.generated_at = "2026-04-05T09:00:00Z".to_string();

        if let Some(item) = snapshot
            .message_groups
            .iter_mut()
            .flat_map(|group| group.items.iter_mut())
            .find(|item| item.id == "msg_github_security")
        {
            item.received_at = "2026-04-01T08:58:00Z".to_string();
        }

        if let Some(detail) = snapshot
            .message_details
            .iter_mut()
            .find(|detail| detail.id == "msg_github_security")
        {
            detail.received_at = "2026-04-01T08:58:00Z".to_string();
        }

        let repository = InMemoryWorkspaceRepository {
            snapshot: RefCell::new(Some(snapshot)),
        };
        let filter = WorkspaceMessageFilter {
            verification_only: true,
            recent_hours: Some(48),
            ..WorkspaceMessageFilter::default()
        };

        let messages = list_workspace_messages(&repository, &filter).expect("48 小时过滤应成功");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, "msg_linear_verify");
    }

    #[test]
    fn resolves_current_site_context_with_exact_match_and_candidates() {
        let repository = InMemoryWorkspaceRepository::default();

        let exact = resolve_workspace_site_context(&repository, "https://www.github.com/login")
            .expect("站点解析应成功");
        let candidate_only =
            resolve_workspace_site_context(&repository, "lin").expect("候选站点解析应成功");

        assert_eq!(exact.normalized_domain.as_deref(), Some("github.com"));
        assert_eq!(
            exact
                .matched_site
                .as_ref()
                .map(|site| site.hostname.as_str()),
            Some("github.com")
        );
        assert!(exact.candidate_sites.is_empty());
        assert!(candidate_only.matched_site.is_none());
        assert_eq!(
            candidate_only
                .candidate_sites
                .first()
                .map(|site| site.hostname.as_str()),
            Some("linear.app")
        );
    }

    #[test]
    fn reads_message_detail_from_cache() {
        let repository = InMemoryWorkspaceRepository::default();

        let detail =
            read_workspace_message(&repository, "msg_github_security").expect("读取消息详情应成功");

        assert_eq!(detail.account_id, "acct_primary-example-com");
        assert!(detail.prefetched_body);
    }

    #[test]
    fn opens_message_by_marking_it_read_without_processing_it() {
        let repository = InMemoryWorkspaceRepository::default();

        let result =
            open_workspace_message(&repository, "msg_github_security").expect("打开消息应成功");
        let inbox_mailbox = result
            .snapshot
            .mailboxes
            .iter()
            .find(|mailbox| mailbox.id == "acct_primary-example-com/inbox")
            .expect("应存在 Inbox 邮箱");

        assert_eq!(result.detail.read_state, MessageReadState::Read);
        assert_eq!(result.detail.status, MessageStatus::Pending);
        assert_eq!(inbox_mailbox.unread_count, 1);
    }

    #[test]
    fn opens_original_message_and_marks_it_read() {
        let repository = InMemoryWorkspaceRepository::default();

        let result = open_workspace_message_original(&repository, "msg_github_security")
            .expect("打开原始邮件应成功");

        assert!(result.original_url.contains("msg_github_security"));
        assert!(
            result
                .snapshot
                .message_details
                .iter()
                .any(|detail| detail.id == "msg_github_security"
                    && detail.read_state == MessageReadState::Read)
        );
    }

    #[test]
    fn confirms_site_even_when_no_messages_have_matched_it_yet() {
        let repository = InMemoryWorkspaceRepository::default();

        let snapshot = confirm_workspace_site(&repository, "https://vercel.com/login", None)
            .expect("确认站点应成功");
        let site = snapshot
            .site_summaries
            .iter()
            .find(|site| site.hostname == "vercel.com")
            .expect("应新增 vercel.com 站点");

        assert_eq!(site.pending_count, 0);
        assert_eq!(site.label, "vercel.com");
    }

    #[test]
    fn updates_message_status_and_rebuilds_workspace_snapshot() {
        let repository = InMemoryWorkspaceRepository::default();

        let snapshot = update_workspace_message_status(
            &repository,
            "msg_github_security",
            MessageStatus::Processed,
        )
        .expect("更新消息状态应成功");

        let processed_group = snapshot
            .message_groups
            .iter()
            .find(|group| group.id == "processed")
            .expect("应存在已处理分组");
        let github_site = snapshot
            .site_summaries
            .iter()
            .find(|site| site.hostname == "github.com")
            .expect("应存在 GitHub 站点");

        assert_eq!(snapshot.selected_message.status, MessageStatus::Processed);
        assert!(processed_group.items.iter().any(
            |item| item.id == "msg_github_security" && item.status == MessageStatus::Processed
        ));
        assert_eq!(github_site.pending_count, 0);
    }

    #[test]
    fn updates_message_read_state_without_processing_message() {
        let repository = InMemoryWorkspaceRepository::default();

        let snapshot = update_workspace_message_read_state(
            &repository,
            "msg_github_security",
            MessageReadState::Read,
        )
        .expect("更新消息已读状态应成功");
        let inbox_mailbox = snapshot
            .mailboxes
            .iter()
            .find(|mailbox| mailbox.id == "acct_primary-example-com/inbox")
            .expect("应存在 Inbox 邮箱");

        assert_eq!(snapshot.selected_message.read_state, MessageReadState::Read);
        assert_eq!(snapshot.selected_message.status, MessageStatus::Pending);
        assert!(
            snapshot
                .message_details
                .iter()
                .any(|detail| detail.id == "msg_github_security"
                    && detail.read_state == MessageReadState::Read)
        );
        assert_eq!(inbox_mailbox.unread_count, 1);
    }

    #[test]
    fn applies_copy_code_action_by_marking_processed_and_removing_extract() {
        let repository = InMemoryWorkspaceRepository::default();

        let result = apply_workspace_message_action(
            &repository,
            "msg_github_security",
            WorkspaceMessageAction::CopyCode,
        )
        .expect("复制验证码动作应成功");

        assert_eq!(result.copied_value.as_deref(), Some("362149"));
        assert_eq!(
            result.snapshot.selected_message.status,
            MessageStatus::Processed
        );
        assert!(
            result
                .snapshot
                .extracts
                .iter()
                .all(|extract| extract.id != "extract_github_code")
        );
    }

    #[test]
    fn applies_open_link_action_by_marking_processed_and_removing_extract() {
        let repository = InMemoryWorkspaceRepository::default();

        let result = apply_workspace_message_action(
            &repository,
            "msg_linear_verify",
            WorkspaceMessageAction::OpenLink,
        )
        .expect("打开验证链接动作应成功");

        assert_eq!(
            result.opened_url.as_deref(),
            Some("https://linear.app/login")
        );
        assert!(
            result
                .snapshot
                .message_details
                .iter()
                .any(|detail| detail.id == "msg_linear_verify"
                    && detail.status == MessageStatus::Processed)
        );
        assert!(
            result
                .snapshot
                .extracts
                .iter()
                .all(|extract| extract.id != "extract_linear_link")
        );
    }

    fn sample_account(display_name: &str) -> AccountSummary {
        AccountSummary {
            id: "acct_primary-example-com".to_string(),
            display_name: display_name.to_string(),
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

    fn sample_processing_snapshot(app_name: &str) -> WorkspaceBootstrapSnapshot {
        WorkspaceBootstrapSnapshot {
            app_name: app_name.to_string(),
            generated_at: "2026-04-05T09:00:00Z".to_string(),
            default_view: WorkspaceViewId::RecentVerification,
            navigation: vec![
                NavigationItem {
                    id: WorkspaceViewId::RecentVerification,
                    label: "Recent verification".to_string(),
                    badge: 2,
                },
                NavigationItem {
                    id: WorkspaceViewId::AllInbox,
                    label: "All inbox".to_string(),
                    badge: 3,
                },
                NavigationItem {
                    id: WorkspaceViewId::SiteList,
                    label: "Sites".to_string(),
                    badge: 3,
                },
                NavigationItem {
                    id: WorkspaceViewId::Accounts,
                    label: "Accounts".to_string(),
                    badge: 1,
                },
            ],
            mailboxes: vec![
                WorkspaceMailboxSummary {
                    id: "acct_primary-example-com/inbox".to_string(),
                    account_id: "acct_primary-example-com".to_string(),
                    account_name: "Primary Gmail".to_string(),
                    label: "Inbox".to_string(),
                    kind: WorkspaceMailboxKind::Inbox,
                    total_count: 2,
                    unread_count: 2,
                    verification_count: 2,
                },
                WorkspaceMailboxSummary {
                    id: "acct_primary-example-com/spam-junk".to_string(),
                    account_id: "acct_primary-example-com".to_string(),
                    account_name: "Primary Gmail".to_string(),
                    label: "Spam/Junk".to_string(),
                    kind: WorkspaceMailboxKind::SpamJunk,
                    total_count: 1,
                    unread_count: 0,
                    verification_count: 1,
                },
            ],
            message_groups: vec![
                WorkspaceMessageGroup {
                    id: "pending".to_string(),
                    label: "待处理".to_string(),
                    items: vec![
                        WorkspaceMessageItem {
                            id: "msg_github_security".to_string(),
                            account_id: "acct_primary-example-com".to_string(),
                            subject: "GitHub 安全验证码".to_string(),
                            sender: "noreply@github.com".to_string(),
                            account_name: "Primary Gmail".to_string(),
                            mailbox_id: "acct_primary-example-com/inbox".to_string(),
                            mailbox_label: "Inbox".to_string(),
                            received_at: "2026-04-05T08:58:00Z".to_string(),
                            category: MessageCategory::Security,
                            status: MessageStatus::Pending,
                            read_state: MessageReadState::Unread,
                            has_code: true,
                            has_link: true,
                            preview: "你的 GitHub 登录验证码是 362149。".to_string(),
                            prefetched_body: true,
                            synced_at: "2026-04-05T09:00:00Z".to_string(),
                        },
                        WorkspaceMessageItem {
                            id: "msg_linear_verify".to_string(),
                            account_id: "acct_primary-example-com".to_string(),
                            subject: "Linear 验证链接".to_string(),
                            sender: "hello@linear.app".to_string(),
                            account_name: "Primary Gmail".to_string(),
                            mailbox_id: "acct_primary-example-com/inbox".to_string(),
                            mailbox_label: "Inbox".to_string(),
                            received_at: "2026-04-05T08:41:00Z".to_string(),
                            category: MessageCategory::Registration,
                            status: MessageStatus::Pending,
                            read_state: MessageReadState::Unread,
                            has_code: false,
                            has_link: true,
                            preview: "点击邮件里的安全链接完成登录。".to_string(),
                            prefetched_body: true,
                            synced_at: "2026-04-05T09:00:00Z".to_string(),
                        },
                    ],
                },
                WorkspaceMessageGroup {
                    id: "processed".to_string(),
                    label: "已处理".to_string(),
                    items: vec![WorkspaceMessageItem {
                        id: "msg_notion_welcome".to_string(),
                        account_id: "acct_primary-example-com".to_string(),
                        subject: "Notion 欢迎邮件".to_string(),
                        sender: "team@makenotion.com".to_string(),
                        account_name: "Primary Gmail".to_string(),
                        mailbox_id: "acct_primary-example-com/spam-junk".to_string(),
                        mailbox_label: "Spam/Junk".to_string(),
                        received_at: "2026-04-02T07:12:00Z".to_string(),
                        category: MessageCategory::Marketing,
                        status: MessageStatus::Processed,
                        read_state: MessageReadState::Read,
                        has_code: false,
                        has_link: true,
                        preview: "欢迎继续完成 Notion 设置。".to_string(),
                        prefetched_body: true,
                        synced_at: "2026-04-05T09:00:00Z".to_string(),
                    }],
                },
            ],
            selected_message: WorkspaceMessageDetail {
                id: "msg_github_security".to_string(),
                account_id: "acct_primary-example-com".to_string(),
                subject: "GitHub 安全验证码".to_string(),
                sender: "noreply@github.com".to_string(),
                account_name: "Primary Gmail".to_string(),
                mailbox_id: "acct_primary-example-com/inbox".to_string(),
                mailbox_label: "Inbox".to_string(),
                received_at: "2026-04-05T08:58:00Z".to_string(),
                category: MessageCategory::Security,
                status: MessageStatus::Pending,
                read_state: MessageReadState::Unread,
                site_hint: "github.com".to_string(),
                summary: "你的 GitHub 登录验证码是 362149。".to_string(),
                extracted_code: Some("362149".to_string()),
                verification_link: Some("https://github.com/login/device".to_string()),
                original_message_url: Some(
                    "https://mail.google.com/mail/u/0/#inbox/msg_github_security".to_string(),
                ),
                body_text: Some("你的 GitHub 登录验证码是 362149。".to_string()),
                prefetched_body: true,
                synced_at: "2026-04-05T09:00:00Z".to_string(),
            },
            message_details: vec![
                WorkspaceMessageDetail {
                    id: "msg_github_security".to_string(),
                    account_id: "acct_primary-example-com".to_string(),
                    subject: "GitHub 安全验证码".to_string(),
                    sender: "noreply@github.com".to_string(),
                    account_name: "Primary Gmail".to_string(),
                    mailbox_id: "acct_primary-example-com/inbox".to_string(),
                    mailbox_label: "Inbox".to_string(),
                    received_at: "2026-04-05T08:58:00Z".to_string(),
                    category: MessageCategory::Security,
                    status: MessageStatus::Pending,
                    read_state: MessageReadState::Unread,
                    site_hint: "github.com".to_string(),
                    summary: "你的 GitHub 登录验证码是 362149。".to_string(),
                    extracted_code: Some("362149".to_string()),
                    verification_link: Some("https://github.com/login/device".to_string()),
                    original_message_url: Some(
                        "https://mail.google.com/mail/u/0/#inbox/msg_github_security".to_string(),
                    ),
                    body_text: Some("你的 GitHub 登录验证码是 362149。".to_string()),
                    prefetched_body: true,
                    synced_at: "2026-04-05T09:00:00Z".to_string(),
                },
                WorkspaceMessageDetail {
                    id: "msg_linear_verify".to_string(),
                    account_id: "acct_primary-example-com".to_string(),
                    subject: "Linear 验证链接".to_string(),
                    sender: "hello@linear.app".to_string(),
                    account_name: "Primary Gmail".to_string(),
                    mailbox_id: "acct_primary-example-com/inbox".to_string(),
                    mailbox_label: "Inbox".to_string(),
                    received_at: "2026-04-05T08:41:00Z".to_string(),
                    category: MessageCategory::Registration,
                    status: MessageStatus::Pending,
                    read_state: MessageReadState::Unread,
                    site_hint: "linear.app".to_string(),
                    summary: "点击邮件里的安全链接完成登录。".to_string(),
                    extracted_code: None,
                    verification_link: Some("https://linear.app/login".to_string()),
                    original_message_url: Some(
                        "https://outlook.office.com/mail/id/msg_linear_verify".to_string(),
                    ),
                    body_text: Some("点击邮件里的安全链接完成登录。".to_string()),
                    prefetched_body: true,
                    synced_at: "2026-04-05T09:00:00Z".to_string(),
                },
                WorkspaceMessageDetail {
                    id: "msg_notion_welcome".to_string(),
                    account_id: "acct_primary-example-com".to_string(),
                    subject: "Notion 欢迎邮件".to_string(),
                    sender: "team@makenotion.com".to_string(),
                    account_name: "Primary Gmail".to_string(),
                    mailbox_id: "acct_primary-example-com/spam-junk".to_string(),
                    mailbox_label: "Spam/Junk".to_string(),
                    received_at: "2026-04-02T07:12:00Z".to_string(),
                    category: MessageCategory::Marketing,
                    status: MessageStatus::Processed,
                    read_state: MessageReadState::Read,
                    site_hint: "notion.so".to_string(),
                    summary: "欢迎继续完成 Notion 设置。".to_string(),
                    extracted_code: None,
                    verification_link: Some("https://www.notion.so".to_string()),
                    original_message_url: Some(
                        "https://mail.google.com/mail/u/0/#spam/msg_notion_welcome".to_string(),
                    ),
                    body_text: Some("欢迎继续完成 Notion 设置。".to_string()),
                    prefetched_body: true,
                    synced_at: "2026-04-05T09:00:00Z".to_string(),
                },
            ],
            extracts: vec![
                WorkspaceExtractItem {
                    id: "extract_github_code".to_string(),
                    sender: "GitHub".to_string(),
                    kind: WorkspaceExtractKind::Code,
                    value: "362149".to_string(),
                    label: String::new(),
                    progress_percent: 84,
                    expires_label: "8m".to_string(),
                },
                WorkspaceExtractItem {
                    id: "extract_linear_link".to_string(),
                    sender: "Linear".to_string(),
                    kind: WorkspaceExtractKind::Link,
                    value: "https://linear.app/login".to_string(),
                    label: "Open login link".to_string(),
                    progress_percent: 42,
                    expires_label: "4m".to_string(),
                },
            ],
            site_summaries: vec![
                WorkspaceSiteSummary {
                    id: "site_github".to_string(),
                    label: "GitHub".to_string(),
                    hostname: "github.com".to_string(),
                    pending_count: 1,
                    latest_sender: "noreply@github.com".to_string(),
                },
                WorkspaceSiteSummary {
                    id: "site_linear".to_string(),
                    label: "Linear".to_string(),
                    hostname: "linear.app".to_string(),
                    pending_count: 1,
                    latest_sender: "hello@linear.app".to_string(),
                },
                WorkspaceSiteSummary {
                    id: "site_notion".to_string(),
                    label: "Notion".to_string(),
                    hostname: "notion.so".to_string(),
                    pending_count: 0,
                    latest_sender: "team@makenotion.com".to_string(),
                },
            ],
            sync_status: Some(WorkspaceSyncStatus {
                state: WorkspaceSyncState::Ready,
                summary: "已同步 1 个账号，共 3 封邮件".to_string(),
                phase: Some(WorkspaceSyncPhase::First),
                poll_interval_minutes: Some(3),
                retention_days: Some(30),
                next_poll_at: Some("2026-04-05T09:03:00Z".to_string()),
                folders: vec!["Inbox".to_string(), "Spam/Junk".to_string()],
            }),
        }
    }
}
