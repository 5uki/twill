use crate::domain::account::{AccountSummary, MailServerConfig};
use crate::domain::compose::{
    ComposeMode, MessageDeliveryMode, MessageSendStatus, PrepareComposeInput, PreparedComposeDraft,
    SendMessageInput, SendMessageResult,
};
use crate::domain::error::AppError;
use crate::services::account_service::{AccountRepository, AccountSecretStore};
use crate::services::workspace_service::{self, WorkspaceSnapshotRepository};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageDeliveryRequest {
    pub account_id: String,
    pub login: String,
    pub password: String,
    pub smtp: MailServerConfig,
    pub to: String,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageDeliveryReceipt {
    pub delivery_mode: MessageDeliveryMode,
    pub summary: String,
    pub smtp_endpoint: String,
}

pub trait MessageDeliveryClient {
    fn send_message(
        &self,
        request: &MessageDeliveryRequest,
    ) -> Result<MessageDeliveryReceipt, AppError>;
}

pub fn prepare_compose_draft<R>(
    repository: &R,
    input: PrepareComposeInput,
) -> Result<PreparedComposeDraft, AppError>
where
    R: WorkspaceSnapshotRepository,
{
    let source_message_id = input
        .source_message_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let account_id = input
        .account_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    match input.mode {
        ComposeMode::New => Ok(PreparedComposeDraft {
            mode: ComposeMode::New,
            account_id: account_id.unwrap_or_default(),
            to: String::new(),
            subject: String::new(),
            body: String::new(),
            source_message_id: None,
        }),
        ComposeMode::Reply | ComposeMode::Forward => {
            let source_message_id = source_message_id.ok_or_else(|| AppError::Validation {
                field: "source_message_id".to_string(),
                message: "reply / forward 需要提供来源消息 ID".to_string(),
            })?;
            let detail = workspace_service::read_workspace_message(repository, &source_message_id)?;
            let subject = build_compose_subject(input.mode, &detail.subject);
            let body = build_compose_body(input.mode, &detail);
            let to = if input.mode == ComposeMode::Reply {
                detail.sender.clone()
            } else {
                String::new()
            };

            Ok(PreparedComposeDraft {
                mode: input.mode,
                account_id: detail.account_id,
                to,
                subject,
                body,
                source_message_id: Some(source_message_id),
            })
        }
    }
}

pub fn send_message<R, S, D>(
    repository: &R,
    secret_store: &S,
    delivery_client: &D,
    input: SendMessageInput,
) -> Result<SendMessageResult, AppError>
where
    R: AccountRepository,
    S: AccountSecretStore,
    D: MessageDeliveryClient,
{
    let input = sanitize_send_message_input(input);

    validate_send_message_input(&input)?;
    let account = find_account(repository, &input.account_id)?;
    let password = secret_store
        .read_secret(&account.id)?
        .ok_or_else(|| AppError::Validation {
            field: "account.credential".to_string(),
            message: format!("账号 {} 缺少系统安全存储密码，暂时无法发送邮件", account.id),
        })?;
    let receipt = delivery_client.send_message(&MessageDeliveryRequest {
        account_id: account.id.clone(),
        login: account.login.clone(),
        password,
        smtp: account.smtp.clone(),
        to: input.to.clone(),
        subject: input.subject.clone(),
        body: input.body.clone(),
    })?;

    Ok(SendMessageResult {
        account_id: account.id,
        to: input.to,
        subject: input.subject,
        status: MessageSendStatus::Sent,
        delivery_mode: receipt.delivery_mode,
        summary: receipt.summary,
        smtp_endpoint: receipt.smtp_endpoint,
    })
}

fn build_compose_subject(mode: ComposeMode, subject: &str) -> String {
    let prefix = match mode {
        ComposeMode::New => return subject.trim().to_string(),
        ComposeMode::Reply => "Re:",
        ComposeMode::Forward => "Fwd:",
    };
    let trimmed_subject = subject.trim();

    if trimmed_subject
        .get(..prefix.len())
        .is_some_and(|value| value.eq_ignore_ascii_case(prefix))
    {
        trimmed_subject.to_string()
    } else {
        format!("{prefix} {trimmed_subject}")
    }
}

fn build_compose_body(
    mode: ComposeMode,
    detail: &crate::domain::workspace::WorkspaceMessageDetail,
) -> String {
    let original_body = detail
        .body_text
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(detail.summary.as_str());

    match mode {
        ComposeMode::New => String::new(),
        ComposeMode::Reply => {
            let quoted_body = original_body
                .lines()
                .map(|line| format!("> {line}"))
                .collect::<Vec<_>>()
                .join("\n");

            format!(
                "\n\n在 {}，{} 写道：\n{}",
                detail.received_at, detail.sender, quoted_body
            )
        }
        ComposeMode::Forward => format!(
            "\n\n---------- 转发邮件 ----------\n发件人: {}\n账号: {}\n时间: {}\n主题: {}\n\n{}",
            detail.sender, detail.account_name, detail.received_at, detail.subject, original_body
        ),
    }
}

fn sanitize_send_message_input(input: SendMessageInput) -> SendMessageInput {
    SendMessageInput {
        account_id: input.account_id.trim().to_string(),
        to: input.to.trim().to_string(),
        subject: input.subject.trim().to_string(),
        body: input.body.trim().to_string(),
    }
}

fn validate_send_message_input(input: &SendMessageInput) -> Result<(), AppError> {
    if input.account_id.is_empty() {
        return Err(AppError::Validation {
            field: "account_id".to_string(),
            message: "请选择用于发信的账号".to_string(),
        });
    }

    if input.to.is_empty() || !input.to.contains('@') || input.to.contains(char::is_whitespace) {
        return Err(AppError::Validation {
            field: "to".to_string(),
            message: "请输入有效的收件人邮箱地址".to_string(),
        });
    }

    if input.subject.is_empty() {
        return Err(AppError::Validation {
            field: "subject".to_string(),
            message: "请输入邮件主题".to_string(),
        });
    }

    if input.body.is_empty() {
        return Err(AppError::Validation {
            field: "body".to_string(),
            message: "请输入邮件正文".to_string(),
        });
    }

    Ok(())
}

fn find_account<R>(repository: &R, account_id: &str) -> Result<AccountSummary, AppError>
where
    R: AccountRepository,
{
    repository
        .list_accounts()?
        .into_iter()
        .find(|account| account.id == account_id)
        .ok_or_else(|| AppError::Validation {
            field: "account_id".to_string(),
            message: format!("未找到账号 {account_id}"),
        })
}

#[cfg(test)]
mod tests {
    use super::{
        MessageDeliveryClient, MessageDeliveryReceipt, MessageDeliveryRequest,
        prepare_compose_draft, send_message,
    };
    use crate::domain::account::{
        AccountCredentialState, AccountSummary, MailSecurity, MailServerConfig,
    };
    use crate::domain::compose::{
        ComposeMode, MessageDeliveryMode, PrepareComposeInput, SendMessageInput,
    };
    use crate::domain::error::AppError;
    use crate::services::account_service::{AccountRepository, AccountSecretStore};
    use crate::services::workspace_service::WorkspaceSnapshotRepository;
    use std::cell::RefCell;

    #[derive(Default)]
    struct InMemoryAccountRepository {
        accounts: RefCell<Vec<AccountSummary>>,
    }

    impl AccountRepository for InMemoryAccountRepository {
        fn list_accounts(&self) -> Result<Vec<AccountSummary>, AppError> {
            Ok(self.accounts.borrow().clone())
        }

        fn save_account(&self, _account: &AccountSummary) -> Result<(), AppError> {
            unimplemented!("compose_service tests do not save accounts")
        }

        fn delete_account(&self, _account_id: &str) -> Result<(), AppError> {
            unimplemented!("compose_service tests do not delete accounts")
        }
    }

    #[derive(Default)]
    struct InMemorySecretStore {
        secret: RefCell<Option<String>>,
    }

    impl AccountSecretStore for InMemorySecretStore {
        fn save_secret(&self, _account_id: &str, _secret: &str) -> Result<(), AppError> {
            unimplemented!("compose_service tests do not save secrets")
        }

        fn read_secret(&self, _account_id: &str) -> Result<Option<String>, AppError> {
            Ok(self.secret.borrow().clone())
        }

        fn delete_secret(&self, _account_id: &str) -> Result<(), AppError> {
            unimplemented!("compose_service tests do not delete secrets")
        }

        fn has_secret(&self, _account_id: &str) -> Result<bool, AppError> {
            Ok(self.secret.borrow().is_some())
        }
    }

    struct InMemoryWorkspaceRepository {
        snapshot: RefCell<Option<crate::domain::workspace::WorkspaceBootstrapSnapshot>>,
    }

    impl Default for InMemoryWorkspaceRepository {
        fn default() -> Self {
            Self {
                snapshot: RefCell::new(Some(
                    crate::services::workspace_service::tests::sample_processing_snapshot(
                        "Workspace",
                    ),
                )),
            }
        }
    }

    impl WorkspaceSnapshotRepository for InMemoryWorkspaceRepository {
        fn load_snapshot(
            &self,
        ) -> Result<Option<crate::domain::workspace::WorkspaceBootstrapSnapshot>, AppError>
        {
            Ok(self.snapshot.borrow().clone())
        }

        fn save_snapshot(
            &self,
            snapshot: &crate::domain::workspace::WorkspaceBootstrapSnapshot,
        ) -> Result<(), AppError> {
            *self.snapshot.borrow_mut() = Some(snapshot.clone());
            Ok(())
        }
    }

    #[derive(Default)]
    struct RecordingDeliveryClient {
        requests: RefCell<Vec<MessageDeliveryRequest>>,
    }

    impl MessageDeliveryClient for RecordingDeliveryClient {
        fn send_message(
            &self,
            request: &MessageDeliveryRequest,
        ) -> Result<MessageDeliveryReceipt, AppError> {
            self.requests.borrow_mut().push(request.clone());

            Ok(MessageDeliveryReceipt {
                delivery_mode: MessageDeliveryMode::Simulated,
                summary: "模拟发送已提交".to_string(),
                smtp_endpoint: format!("{}:{}", request.smtp.host, request.smtp.port),
            })
        }
    }

    #[test]
    fn sends_message_with_trimmed_input_and_resolved_account_credentials() {
        let repository = InMemoryAccountRepository::default();
        repository.accounts.borrow_mut().push(sample_account());
        let secret_store = InMemorySecretStore::default();
        *secret_store.secret.borrow_mut() = Some("app-password".to_string());
        let delivery_client = RecordingDeliveryClient::default();

        let result = send_message(
            &repository,
            &secret_store,
            &delivery_client,
            SendMessageInput {
                account_id: "  acct_primary-example-com  ".to_string(),
                to: "  dev@example.com  ".to_string(),
                subject: "  Launch update  ".to_string(),
                body: "  Shipping today.  ".to_string(),
            },
        )
        .expect("发送消息应成功");

        assert_eq!(result.account_id, "acct_primary-example-com");
        assert_eq!(result.to, "dev@example.com");
        assert_eq!(result.subject, "Launch update");
        assert_eq!(result.delivery_mode, MessageDeliveryMode::Simulated);
        assert_eq!(delivery_client.requests.borrow().len(), 1);
        assert_eq!(
            delivery_client.requests.borrow()[0].password,
            "app-password"
        );
        assert_eq!(delivery_client.requests.borrow()[0].body, "Shipping today.");
    }

    #[test]
    fn prepares_reply_draft_from_workspace_message() {
        let repository = InMemoryWorkspaceRepository::default();

        let draft = prepare_compose_draft(
            &repository,
            PrepareComposeInput {
                mode: ComposeMode::Reply,
                source_message_id: Some("msg_github_security".to_string()),
                account_id: None,
            },
        )
        .expect("reply 草稿预填应成功");

        assert_eq!(draft.account_id, "acct_primary-example-com");
        assert_eq!(draft.to, "noreply@github.com");
        assert_eq!(draft.subject, "Re: GitHub 安全验证码");
        assert!(
            draft
                .body
                .contains("在 2026-04-05T08:58:00Z，noreply@github.com 写道："),
            "reply 应包含来源邮件引用头"
        );
        assert!(draft.body.contains("> 你的 GitHub 登录验证码是 362149。"));
    }

    #[test]
    fn prepares_forward_draft_without_duplicate_prefix() {
        let repository = InMemoryWorkspaceRepository::default();
        let mut snapshot = repository
            .load_snapshot()
            .expect("读取快照应成功")
            .expect("应存在默认快照");
        snapshot.message_details[0].subject = "Fwd: GitHub 安全验证码".to_string();
        repository
            .save_snapshot(&snapshot)
            .expect("写回测试快照应成功");

        let draft = prepare_compose_draft(
            &repository,
            PrepareComposeInput {
                mode: ComposeMode::Forward,
                source_message_id: Some("msg_github_security".to_string()),
                account_id: None,
            },
        )
        .expect("forward 草稿预填应成功");

        assert_eq!(draft.to, "");
        assert_eq!(draft.subject, "Fwd: GitHub 安全验证码");
        assert!(draft.body.contains("---------- 转发邮件 ----------"));
        assert!(draft.body.contains("发件人: noreply@github.com"));
    }

    #[test]
    fn rejects_reply_prepare_without_source_message() {
        let repository = InMemoryWorkspaceRepository::default();

        let error = prepare_compose_draft(
            &repository,
            PrepareComposeInput {
                mode: ComposeMode::Reply,
                source_message_id: None,
                account_id: None,
            },
        )
        .expect_err("reply 缺少来源消息时必须报错");

        assert_eq!(
            error,
            AppError::Validation {
                field: "source_message_id".to_string(),
                message: "reply / forward 需要提供来源消息 ID".to_string(),
            }
        );
    }

    #[test]
    fn rejects_invalid_recipient_before_touching_delivery_client() {
        let repository = InMemoryAccountRepository::default();
        repository.accounts.borrow_mut().push(sample_account());
        let secret_store = InMemorySecretStore::default();
        *secret_store.secret.borrow_mut() = Some("app-password".to_string());
        let delivery_client = RecordingDeliveryClient::default();

        let error = send_message(
            &repository,
            &secret_store,
            &delivery_client,
            SendMessageInput {
                account_id: "acct_primary-example-com".to_string(),
                to: "not-an-email".to_string(),
                subject: "Hello".to_string(),
                body: "World".to_string(),
            },
        )
        .expect_err("非法收件人必须报错");

        assert_eq!(
            error,
            AppError::Validation {
                field: "to".to_string(),
                message: "请输入有效的收件人邮箱地址".to_string(),
            }
        );
        assert!(delivery_client.requests.borrow().is_empty());
    }

    #[test]
    fn rejects_message_send_when_account_secret_is_missing() {
        let repository = InMemoryAccountRepository::default();
        repository.accounts.borrow_mut().push(sample_account());
        let secret_store = InMemorySecretStore::default();
        let delivery_client = RecordingDeliveryClient::default();

        let error = send_message(
            &repository,
            &secret_store,
            &delivery_client,
            SendMessageInput {
                account_id: "acct_primary-example-com".to_string(),
                to: "dev@example.com".to_string(),
                subject: "Hello".to_string(),
                body: "World".to_string(),
            },
        )
        .expect_err("缺少系统密码时必须拒绝发送");

        assert_eq!(
            error,
            AppError::Validation {
                field: "account.credential".to_string(),
                message: "账号 acct_primary-example-com 缺少系统安全存储密码，暂时无法发送邮件"
                    .to_string(),
            }
        );
        assert!(delivery_client.requests.borrow().is_empty());
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
}
