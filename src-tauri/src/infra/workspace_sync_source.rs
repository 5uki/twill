use crate::domain::account::AccountSummary;
use crate::domain::error::AppError;
use crate::domain::workspace::{
    NavigationItem, WorkspaceBootstrapSnapshot, WorkspaceMailboxKind, WorkspaceMailboxSummary,
    WorkspaceMessageDetail, WorkspaceMessageGroup, WorkspaceSyncPhase, WorkspaceSyncState,
    WorkspaceSyncStatus, WorkspaceViewId,
};
use crate::infra::static_workspace;
use crate::services::workspace_service::WorkspaceSyncSource;
use std::collections::{BTreeMap, btree_map::Entry};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default)]
pub struct SeededWorkspaceSyncSource;

impl WorkspaceSyncSource for SeededWorkspaceSyncSource {
    fn build_snapshot(
        &self,
        accounts: &[AccountSummary],
        previous_snapshot: Option<&WorkspaceBootstrapSnapshot>,
    ) -> Result<WorkspaceBootstrapSnapshot, AppError> {
        let mut snapshot = static_workspace::load_snapshot();
        let generated_at = current_sync_timestamp();
        let assignments = assign_mailboxes_and_accounts(&mut snapshot, accounts, &generated_at);
        update_message_details(&mut snapshot, &assignments);
        update_selected_message(
            &mut snapshot.selected_message,
            &snapshot.message_details,
            accounts,
        );
        snapshot.mailboxes = build_mailboxes(&snapshot.message_groups);
        update_navigation(
            &mut snapshot.navigation,
            &snapshot.message_groups,
            snapshot.site_summaries.len(),
            accounts.len(),
        );
        snapshot.generated_at = generated_at;
        snapshot.sync_status = Some(build_sync_status(
            accounts.len(),
            &snapshot.message_groups,
            previous_snapshot.is_some(),
            &snapshot.generated_at,
        ));

        Ok(snapshot)
    }
}

#[derive(Clone)]
struct MessageAssignment {
    account_id: String,
    account_name: String,
    mailbox_id: String,
    mailbox_label: String,
    prefetched_body: bool,
    synced_at: String,
}

fn assign_mailboxes_and_accounts(
    snapshot: &mut WorkspaceBootstrapSnapshot,
    accounts: &[AccountSummary],
    generated_at: &str,
) -> BTreeMap<String, MessageAssignment> {
    let mut assignments = BTreeMap::new();

    for (index, item) in snapshot
        .message_groups
        .iter_mut()
        .flat_map(|group| group.items.iter_mut())
        .enumerate()
    {
        let account = &accounts[index % accounts.len()];
        let mailbox_kind = choose_mailbox_kind(item);
        let mailbox_label = mailbox_label(mailbox_kind).to_string();
        let mailbox_id = format!("{}/{}", account.id, mailbox_storage_key(mailbox_kind));
        let assignment = MessageAssignment {
            account_id: account.id.clone(),
            account_name: account.display_name.clone(),
            mailbox_id,
            mailbox_label,
            prefetched_body: item.has_code || item.has_link,
            synced_at: generated_at.to_string(),
        };

        item.account_id = assignment.account_id.clone();
        item.account_name = assignment.account_name.clone();
        item.mailbox_id = assignment.mailbox_id.clone();
        item.mailbox_label = assignment.mailbox_label.clone();
        item.prefetched_body = assignment.prefetched_body;
        item.synced_at = assignment.synced_at.clone();
        assignments.insert(item.id.clone(), assignment);
    }

    assignments
}

fn update_message_details(
    snapshot: &mut WorkspaceBootstrapSnapshot,
    assignments: &BTreeMap<String, MessageAssignment>,
) {
    let existing_details = snapshot
        .message_details
        .iter()
        .cloned()
        .map(|detail| (detail.id.clone(), detail))
        .collect::<BTreeMap<_, _>>();

    snapshot.message_details =
        snapshot
            .message_groups
            .iter()
            .flat_map(|group| group.items.iter())
            .map(|item| {
                let mut detail = existing_details.get(&item.id).cloned().unwrap_or_else(|| {
                    WorkspaceMessageDetail {
                        id: item.id.clone(),
                        account_id: item.account_id.clone(),
                        subject: item.subject.clone(),
                        sender: item.sender.clone(),
                        account_name: item.account_name.clone(),
                        mailbox_id: item.mailbox_id.clone(),
                        mailbox_label: item.mailbox_label.clone(),
                        received_at: item.received_at.clone(),
                        category: item.category,
                        status: item.status,
                        read_state: item.read_state,
                        site_hint: item
                            .sender
                            .split('@')
                            .nth(1)
                            .unwrap_or("unknown")
                            .to_string(),
                        summary: item.preview.clone(),
                        extracted_code: None,
                        verification_link: None,
                        original_message_url: None,
                        body_text: None,
                        prefetched_body: item.prefetched_body,
                        synced_at: item.synced_at.clone(),
                    }
                });

                if let Some(assignment) = assignments.get(&item.id) {
                    detail.account_id = assignment.account_id.clone();
                    detail.account_name = assignment.account_name.clone();
                    detail.mailbox_id = assignment.mailbox_id.clone();
                    detail.mailbox_label = assignment.mailbox_label.clone();
                    detail.prefetched_body = assignment.prefetched_body;
                    detail.synced_at = assignment.synced_at.clone();
                }

                if detail.body_text.is_none() && detail.prefetched_body {
                    detail.body_text = Some(format!("{}\n\n{}", detail.subject, detail.summary));
                }

                detail
            })
            .collect();
}

fn update_selected_message(
    selected_message: &mut WorkspaceMessageDetail,
    message_details: &[WorkspaceMessageDetail],
    accounts: &[AccountSummary],
) {
    if let Some(detail) = message_details
        .iter()
        .find(|detail| detail.id == selected_message.id)
        .cloned()
    {
        *selected_message = detail;
        return;
    }

    selected_message.account_id = accounts[0].id.clone();
    selected_message.account_name = accounts[0].display_name.clone();
}

fn update_navigation(
    navigation: &mut [NavigationItem],
    message_groups: &[WorkspaceMessageGroup],
    site_summary_count: usize,
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
            WorkspaceViewId::SiteList => site_summary_count as u32,
            WorkspaceViewId::Accounts => account_count as u32,
        };
    }
}

fn build_sync_status(
    account_count: usize,
    message_groups: &[WorkspaceMessageGroup],
    has_previous_snapshot: bool,
    generated_at: &str,
) -> WorkspaceSyncStatus {
    let message_count = message_groups
        .iter()
        .flat_map(|group| group.items.iter())
        .count();
    let phase = if has_previous_snapshot {
        WorkspaceSyncPhase::Incremental
    } else {
        WorkspaceSyncPhase::First
    };
    let summary = match phase {
        WorkspaceSyncPhase::First => {
            format!("首次同步完成，已同步 {account_count} 个账号，共 {message_count} 封邮件")
        }
        WorkspaceSyncPhase::Incremental => {
            format!("已刷新 {account_count} 个账号，共 {message_count} 封邮件")
        }
    };

    WorkspaceSyncStatus {
        state: WorkspaceSyncState::Ready,
        summary,
        phase: Some(phase),
        poll_interval_minutes: Some(3),
        retention_days: Some(30),
        next_poll_at: Some(next_poll_timestamp(generated_at, 180)),
        folders: vec!["Inbox".to_string(), "Spam/Junk".to_string()],
    }
}

fn current_sync_timestamp() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("系统时间应晚于 epoch")
        .as_secs();

    timestamp.to_string()
}

fn next_poll_timestamp(generated_at: &str, offset_seconds: u64) -> String {
    generated_at
        .parse::<u64>()
        .map(|value| value + offset_seconds)
        .unwrap_or(offset_seconds)
        .to_string()
}

fn choose_mailbox_kind(
    item: &crate::domain::workspace::WorkspaceMessageItem,
) -> WorkspaceMailboxKind {
    if item.category == crate::domain::workspace::MessageCategory::Marketing {
        WorkspaceMailboxKind::SpamJunk
    } else {
        WorkspaceMailboxKind::Inbox
    }
}

fn mailbox_label(kind: WorkspaceMailboxKind) -> &'static str {
    match kind {
        WorkspaceMailboxKind::Inbox => "Inbox",
        WorkspaceMailboxKind::SpamJunk => "Spam/Junk",
    }
}

fn mailbox_storage_key(kind: WorkspaceMailboxKind) -> &'static str {
    match kind {
        WorkspaceMailboxKind::Inbox => "inbox",
        WorkspaceMailboxKind::SpamJunk => "spam-junk",
    }
}

fn build_mailboxes(message_groups: &[WorkspaceMessageGroup]) -> Vec<WorkspaceMailboxSummary> {
    let mut mailboxes = BTreeMap::<(String, WorkspaceMailboxKind), WorkspaceMailboxSummary>::new();

    for item in message_groups.iter().flat_map(|group| group.items.iter()) {
        let kind = parse_mailbox_kind(&item.mailbox_label).unwrap_or(WorkspaceMailboxKind::Inbox);
        match mailboxes.entry((item.account_id.clone(), kind)) {
            Entry::Occupied(mut entry) => {
                let mailbox = entry.get_mut();
                mailbox.total_count += 1;
                mailbox.unread_count +=
                    (item.read_state == crate::domain::workspace::MessageReadState::Unread) as u32;
                mailbox.verification_count += (item.has_code || item.has_link) as u32;
            }
            Entry::Vacant(entry) => {
                entry.insert(WorkspaceMailboxSummary {
                    id: item.mailbox_id.clone(),
                    account_id: item.account_id.clone(),
                    account_name: item.account_name.clone(),
                    label: item.mailbox_label.clone(),
                    kind,
                    total_count: 1,
                    unread_count: (item.read_state
                        == crate::domain::workspace::MessageReadState::Unread)
                        as u32,
                    verification_count: (item.has_code || item.has_link) as u32,
                });
            }
        }
    }

    mailboxes.into_values().collect()
}

fn parse_mailbox_kind(label: &str) -> Option<WorkspaceMailboxKind> {
    match label {
        "Inbox" => Some(WorkspaceMailboxKind::Inbox),
        "Spam/Junk" => Some(WorkspaceMailboxKind::SpamJunk),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::SeededWorkspaceSyncSource;
    use crate::domain::account::{
        AccountCredentialState, AccountSummary, MailSecurity, MailServerConfig,
    };
    use crate::domain::workspace::{WorkspaceMailboxKind, WorkspaceSyncPhase, WorkspaceViewId};
    use crate::services::workspace_service::WorkspaceSyncSource;

    #[test]
    fn rotates_saved_account_names_through_seed_messages() {
        let source = SeededWorkspaceSyncSource;
        let snapshot = source
            .build_snapshot(
                &[
                    sample_account("Primary Gmail", "primary@example.com"),
                    sample_account("Work Outlook", "work@example.com"),
                ],
                None,
            )
            .expect("构建同步快照应成功");

        let account_names = snapshot
            .message_groups
            .iter()
            .flat_map(|group| group.items.iter().map(|item| item.account_name.clone()))
            .collect::<Vec<_>>();

        assert!(account_names.contains(&"Primary Gmail".to_string()));
        assert!(account_names.contains(&"Work Outlook".to_string()));
    }

    #[test]
    fn updates_navigation_badges_from_synced_snapshot() {
        let source = SeededWorkspaceSyncSource;
        let snapshot = source
            .build_snapshot(
                &[sample_account("Primary Gmail", "primary@example.com")],
                None,
            )
            .expect("构建同步快照应成功");
        let account_badge = snapshot
            .navigation
            .iter()
            .find(|item| item.id == WorkspaceViewId::Accounts)
            .expect("导航里必须包含账号入口")
            .badge;

        assert_eq!(account_badge, 1);
    }

    #[test]
    fn adds_sync_status_summary_for_synced_snapshot() {
        let source = SeededWorkspaceSyncSource;
        let snapshot = source
            .build_snapshot(
                &[sample_account("Primary Gmail", "primary@example.com")],
                None,
            )
            .expect("构建同步快照应成功");

        assert_eq!(
            snapshot
                .sync_status
                .as_ref()
                .expect("同步快照必须包含同步状态")
                .summary,
            "首次同步完成，已同步 1 个账号，共 3 封邮件"
        );
        assert_ne!(snapshot.generated_at, "2026-03-29T09:00:00Z");
    }

    #[test]
    fn marks_sync_phase_as_incremental_when_previous_snapshot_exists() {
        let source = SeededWorkspaceSyncSource;
        let previous_snapshot = crate::infra::static_workspace::load_snapshot();
        let snapshot = source
            .build_snapshot(
                &[sample_account("Primary Gmail", "primary@example.com")],
                Some(&previous_snapshot),
            )
            .expect("构建同步快照应成功");

        assert_eq!(
            snapshot.sync_status.and_then(|status| status.phase),
            Some(WorkspaceSyncPhase::Incremental)
        );
    }

    #[test]
    fn builds_mailbox_summaries_for_inbox_and_spam() {
        let source = SeededWorkspaceSyncSource;
        let snapshot = source
            .build_snapshot(
                &[sample_account("Primary Gmail", "primary@example.com")],
                None,
            )
            .expect("构建同步快照应成功");

        assert!(
            snapshot
                .mailboxes
                .iter()
                .any(|mailbox| mailbox.kind == WorkspaceMailboxKind::Inbox)
        );
        assert!(
            snapshot
                .mailboxes
                .iter()
                .any(|mailbox| mailbox.kind == WorkspaceMailboxKind::SpamJunk)
        );
    }

    fn sample_account(display_name: &str, email: &str) -> AccountSummary {
        AccountSummary {
            id: format!("acct_{}", email.replace(['@', '.'], "-")),
            display_name: display_name.to_string(),
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
}
