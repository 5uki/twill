use crate::domain::account::AccountSummary;
use crate::domain::error::AppError;
use crate::domain::workspace::{
    WorkspaceBootstrapSnapshot, WorkspaceMailboxKind, WorkspaceMailboxSummary,
    WorkspaceMessageDetail, WorkspaceMessageItem,
};
use crate::infra::static_workspace;
use crate::services::account_service::AccountRepository;

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

#[derive(Default)]
pub struct WorkspaceMessageFilter {
    pub account_id: Option<String>,
    pub mailbox_kind: Option<WorkspaceMailboxKind>,
    pub verification_only: bool,
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

            !filter.verification_only || item.has_code || item.has_link
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
            message: format!("未找到消息: {message_id}"),
        })
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
    use super::{
        WorkspaceMessageFilter, WorkspaceSnapshotRepository, WorkspaceSyncSource,
        list_workspace_mailboxes, list_workspace_messages, load_workspace_bootstrap,
        read_workspace_message, sync_workspace,
    };
    use crate::domain::account::{
        AccountCredentialState, AccountSummary, MailSecurity, MailServerConfig,
    };
    use crate::domain::error::AppError;
    use crate::domain::workspace::{
        MessageCategory, MessageStatus, NavigationItem, WorkspaceBootstrapSnapshot,
        WorkspaceMailboxKind, WorkspaceMailboxSummary, WorkspaceMessageDetail,
        WorkspaceMessageGroup, WorkspaceMessageItem, WorkspaceSyncPhase, WorkspaceSyncState,
        WorkspaceSyncStatus, WorkspaceViewId,
    };
    use crate::services::account_service::AccountRepository;
    use std::cell::RefCell;

    #[derive(Default)]
    struct InMemoryAccountRepository {
        accounts: RefCell<Vec<AccountSummary>>,
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

    #[derive(Default)]
    struct InMemoryWorkspaceRepository {
        snapshot: RefCell<Option<WorkspaceBootstrapSnapshot>>,
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
        let repository = InMemoryWorkspaceRepository::default();
        let snapshot = load_workspace_bootstrap(&repository).expect("读取工作台快照应成功");

        assert_eq!(snapshot.default_view, WorkspaceViewId::RecentVerification);
    }

    #[test]
    fn prefers_cached_snapshot_over_static_seed() {
        let repository = InMemoryWorkspaceRepository {
            snapshot: RefCell::new(Some(sample_snapshot("缓存快照"))),
        };
        let snapshot = load_workspace_bootstrap(&repository).expect("读取缓存快照应成功");

        assert_eq!(snapshot.app_name, "缓存快照");
    }

    #[test]
    fn falls_back_to_static_seed_when_cache_load_hits_storage_error() {
        struct FailingWorkspaceRepository;

        impl WorkspaceSnapshotRepository for FailingWorkspaceRepository {
            fn load_snapshot(&self) -> Result<Option<WorkspaceBootstrapSnapshot>, AppError> {
                Err(AppError::Storage {
                    message: "缓存目录不可访问".to_string(),
                })
            }

            fn save_snapshot(
                &self,
                _snapshot: &WorkspaceBootstrapSnapshot,
            ) -> Result<(), AppError> {
                Ok(())
            }
        }

        let snapshot =
            load_workspace_bootstrap(&FailingWorkspaceRepository).expect("应回退到静态种子");

        assert_eq!(snapshot.default_view, WorkspaceViewId::RecentVerification);
    }

    #[test]
    fn rejects_sync_when_no_accounts_have_been_saved() {
        let accounts = InMemoryAccountRepository::default();
        let workspace = InMemoryWorkspaceRepository::default();
        let source = FixedWorkspaceSyncSource {
            snapshot: sample_snapshot("Synced"),
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
        let workspace = InMemoryWorkspaceRepository::default();
        let source = FixedWorkspaceSyncSource {
            snapshot: sample_snapshot("Synced"),
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
        let repository = InMemoryWorkspaceRepository {
            snapshot: RefCell::new(Some(sample_snapshot("缓存快照"))),
        };

        let mailboxes = list_workspace_mailboxes(&repository).expect("读取邮箱列表应成功");

        assert_eq!(mailboxes.len(), 2);
        assert_eq!(mailboxes[0].kind, WorkspaceMailboxKind::Inbox);
    }

    #[test]
    fn filters_messages_by_mailbox_and_verification_flag() {
        let repository = InMemoryWorkspaceRepository {
            snapshot: RefCell::new(Some(sample_snapshot("缓存快照"))),
        };
        let filter = WorkspaceMessageFilter {
            account_id: Some("acct_primary-example-com".to_string()),
            mailbox_kind: Some(WorkspaceMailboxKind::Inbox),
            verification_only: true,
        };

        let messages = list_workspace_messages(&repository, &filter).expect("读取消息列表应成功");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, "msg_github_security");
    }

    #[test]
    fn reads_message_detail_from_cache() {
        let repository = InMemoryWorkspaceRepository {
            snapshot: RefCell::new(Some(sample_snapshot("缓存快照"))),
        };

        let detail =
            read_workspace_message(&repository, "msg_github_security").expect("读取消息详情应成功");

        assert_eq!(detail.account_id, "acct_primary-example-com");
        assert!(detail.prefetched_body);
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

    fn sample_snapshot(app_name: &str) -> WorkspaceBootstrapSnapshot {
        WorkspaceBootstrapSnapshot {
            app_name: app_name.to_string(),
            generated_at: "2026-04-05T00:00:00Z".to_string(),
            default_view: WorkspaceViewId::RecentVerification,
            navigation: vec![NavigationItem {
                id: WorkspaceViewId::RecentVerification,
                label: "Recent verification".to_string(),
                badge: 1,
            }],
            mailboxes: vec![
                WorkspaceMailboxSummary {
                    id: "acct_primary-example-com/inbox".to_string(),
                    account_id: "acct_primary-example-com".to_string(),
                    account_name: "Primary Gmail".to_string(),
                    label: "Inbox".to_string(),
                    kind: WorkspaceMailboxKind::Inbox,
                    total_count: 1,
                    unread_count: 1,
                    verification_count: 1,
                },
                WorkspaceMailboxSummary {
                    id: "acct_primary-example-com/spam-junk".to_string(),
                    account_id: "acct_primary-example-com".to_string(),
                    account_name: "Primary Gmail".to_string(),
                    label: "Spam/Junk".to_string(),
                    kind: WorkspaceMailboxKind::SpamJunk,
                    total_count: 1,
                    unread_count: 0,
                    verification_count: 0,
                },
            ],
            message_groups: vec![WorkspaceMessageGroup {
                id: "pending".to_string(),
                label: "待处理".to_string(),
                items: vec![WorkspaceMessageItem {
                    id: "msg_github_security".to_string(),
                    account_id: "acct_primary-example-com".to_string(),
                    subject: "GitHub 安全验证码".to_string(),
                    sender: "noreply@github.com".to_string(),
                    account_name: "Primary Gmail".to_string(),
                    mailbox_id: "acct_primary-example-com/inbox".to_string(),
                    mailbox_label: "Inbox".to_string(),
                    received_at: "2026-04-05T00:00:00Z".to_string(),
                    category: MessageCategory::Security,
                    status: MessageStatus::Pending,
                    has_code: true,
                    has_link: false,
                    preview: "你的 GitHub 登录验证码是 362149。".to_string(),
                    prefetched_body: true,
                    synced_at: "2026-04-05T00:00:00Z".to_string(),
                }],
            }],
            selected_message: WorkspaceMessageDetail {
                id: "msg_github_security".to_string(),
                account_id: "acct_primary-example-com".to_string(),
                subject: "GitHub 安全验证码".to_string(),
                sender: "noreply@github.com".to_string(),
                account_name: "Primary Gmail".to_string(),
                mailbox_id: "acct_primary-example-com/inbox".to_string(),
                mailbox_label: "Inbox".to_string(),
                received_at: "2026-04-05T00:00:00Z".to_string(),
                category: MessageCategory::Security,
                status: MessageStatus::Pending,
                site_hint: "github.com".to_string(),
                summary: "同步后的缓存快照".to_string(),
                extracted_code: Some("362149".to_string()),
                verification_link: None,
                body_text: Some("你的 GitHub 登录验证码是 362149。".to_string()),
                prefetched_body: true,
                synced_at: "2026-04-05T00:00:00Z".to_string(),
            },
            message_details: vec![WorkspaceMessageDetail {
                id: "msg_github_security".to_string(),
                account_id: "acct_primary-example-com".to_string(),
                subject: "GitHub 安全验证码".to_string(),
                sender: "noreply@github.com".to_string(),
                account_name: "Primary Gmail".to_string(),
                mailbox_id: "acct_primary-example-com/inbox".to_string(),
                mailbox_label: "Inbox".to_string(),
                received_at: "2026-04-05T00:00:00Z".to_string(),
                category: MessageCategory::Security,
                status: MessageStatus::Pending,
                site_hint: "github.com".to_string(),
                summary: "同步后的缓存快照".to_string(),
                extracted_code: Some("362149".to_string()),
                verification_link: None,
                body_text: Some("你的 GitHub 登录验证码是 362149。".to_string()),
                prefetched_body: true,
                synced_at: "2026-04-05T00:00:00Z".to_string(),
            }],
            extracts: Vec::new(),
            site_summaries: Vec::new(),
            sync_status: Some(WorkspaceSyncStatus {
                state: WorkspaceSyncState::Ready,
                summary: "已同步 1 个账号，共 1 封邮件".to_string(),
                phase: Some(WorkspaceSyncPhase::First),
                poll_interval_minutes: Some(3),
                retention_days: Some(30),
                next_poll_at: Some("2026-04-05T00:03:00Z".to_string()),
                folders: vec!["Inbox".to_string(), "Spam/Junk".to_string()],
            }),
        }
    }
}
