use crate::domain::account::{
    AccountConnectionTestInput, AccountConnectionTestResult, AccountSummary, AddAccountInput,
};
use crate::domain::error::AppError;

pub trait AccountRepository {
    fn list_accounts(&self) -> Result<Vec<AccountSummary>, AppError>;
    fn save_account(&self, account: &AccountSummary) -> Result<(), AppError>;
}

pub trait AccountConnectionTester {
    fn test_account_connection(
        &self,
        input: &AccountConnectionTestInput,
    ) -> AccountConnectionTestResult;
}

pub fn list_accounts<R>(repository: &R) -> Result<Vec<AccountSummary>, AppError>
where
    R: AccountRepository,
{
    repository.list_accounts()
}

pub fn add_account<R>(repository: &R, input: AddAccountInput) -> Result<AccountSummary, AppError>
where
    R: AccountRepository,
{
    let input = sanitize_add_account_input(input);

    validate_account_identity(&input.display_name, &input.email, &input.login)?;
    validate_mail_server("imap", &input.imap)?;
    validate_mail_server("smtp", &input.smtp)?;

    let existing_accounts = repository.list_accounts()?;

    if existing_accounts
        .iter()
        .any(|account| account.email.eq_ignore_ascii_case(input.email.trim()))
    {
        return Err(AppError::Validation {
            field: "email".to_string(),
            message: "该邮箱地址已经存在".to_string(),
        });
    }

    let account = AccountSummary {
        id: build_account_id(existing_accounts.len() + 1, &input.email),
        display_name: input.display_name.trim().to_string(),
        email: input.email.trim().to_string(),
        login: input.login.trim().to_string(),
        imap: sanitize_server(input.imap),
        smtp: sanitize_server(input.smtp),
    };

    repository.save_account(&account)?;

    Ok(account)
}

pub fn test_account_connection<T>(
    tester: &T,
    input: AccountConnectionTestInput,
) -> Result<AccountConnectionTestResult, AppError>
where
    T: AccountConnectionTester,
{
    let input = sanitize_connection_input(input);

    validate_account_identity(&input.display_name, &input.email, &input.login)?;
    validate_mail_server("imap", &input.imap)?;
    validate_mail_server("smtp", &input.smtp)?;

    Ok(tester.test_account_connection(&input))
}

fn sanitize_connection_input(input: AccountConnectionTestInput) -> AccountConnectionTestInput {
    AccountConnectionTestInput {
        display_name: input.display_name.trim().to_string(),
        email: input.email.trim().to_string(),
        login: input.login.trim().to_string(),
        imap: sanitize_server(input.imap),
        smtp: sanitize_server(input.smtp),
    }
}

fn sanitize_add_account_input(input: AddAccountInput) -> AddAccountInput {
    AddAccountInput {
        display_name: input.display_name.trim().to_string(),
        email: input.email.trim().to_string(),
        login: input.login.trim().to_string(),
        imap: sanitize_server(input.imap),
        smtp: sanitize_server(input.smtp),
    }
}

fn sanitize_server(
    server: crate::domain::account::MailServerConfig,
) -> crate::domain::account::MailServerConfig {
    crate::domain::account::MailServerConfig {
        host: server.host.trim().to_string(),
        port: server.port,
        security: server.security,
    }
}

fn validate_account_identity(display_name: &str, email: &str, login: &str) -> Result<(), AppError> {
    if display_name.trim().is_empty() {
        return Err(AppError::Validation {
            field: "display_name".to_string(),
            message: "账户名称不能为空".to_string(),
        });
    }

    if email.trim().is_empty() || !email.contains('@') {
        return Err(AppError::Validation {
            field: "email".to_string(),
            message: "请输入有效的邮箱地址".to_string(),
        });
    }

    if login.trim().is_empty() {
        return Err(AppError::Validation {
            field: "login".to_string(),
            message: "登录名不能为空".to_string(),
        });
    }

    Ok(())
}

fn validate_mail_server(
    field_prefix: &str,
    server: &crate::domain::account::MailServerConfig,
) -> Result<(), AppError> {
    if server.host.trim().is_empty() {
        return Err(AppError::Validation {
            field: format!("{field_prefix}.host"),
            message: "服务器地址不能为空".to_string(),
        });
    }

    if server.host.contains(char::is_whitespace) || !server.host.contains('.') {
        return Err(AppError::Validation {
            field: format!("{field_prefix}.host"),
            message: "服务器地址格式不合法".to_string(),
        });
    }

    if server.port == 0 {
        return Err(AppError::Validation {
            field: format!("{field_prefix}.port"),
            message: "端口必须大于 0".to_string(),
        });
    }

    Ok(())
}

fn build_account_id(sequence: usize, email: &str) -> String {
    let account_label = email
        .split('@')
        .next()
        .unwrap_or("account")
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    format!("acct_{sequence}_{account_label}")
}

#[cfg(test)]
mod tests {
    use super::{AccountConnectionTester, add_account, list_accounts, test_account_connection};
    use crate::domain::account::{
        AccountConnectionCheck, AccountConnectionCheckTarget, AccountConnectionStatus,
        AccountConnectionTestInput, AccountConnectionTestResult, AccountSummary, AddAccountInput,
        MailSecurity, MailServerConfig,
    };
    use crate::domain::error::AppError;
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
    }

    struct PassingTester;

    impl AccountConnectionTester for PassingTester {
        fn test_account_connection(
            &self,
            _input: &AccountConnectionTestInput,
        ) -> AccountConnectionTestResult {
            AccountConnectionTestResult {
                status: AccountConnectionStatus::Passed,
                summary: "预检通过".to_string(),
                checks: vec![AccountConnectionCheck {
                    target: AccountConnectionCheckTarget::Imap,
                    status: AccountConnectionStatus::Passed,
                    message: "IMAP 组合可继续进入下一步".to_string(),
                }],
            }
        }
    }

    #[test]
    fn adds_account_and_lists_it_back() {
        let repository = InMemoryAccountRepository::default();

        let account = add_account(&repository, sample_add_account_input()).expect("新增账户应成功");
        let accounts = list_accounts(&repository).expect("列出账户应成功");

        assert_eq!(account.id, "acct_1_primary");
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].email, "primary@example.com");
    }

    #[test]
    fn rejects_duplicate_email_accounts() {
        let repository = InMemoryAccountRepository::default();

        add_account(&repository, sample_add_account_input()).expect("首次新增账户应成功");

        let error =
            add_account(&repository, sample_add_account_input()).expect_err("重复邮箱必须报错");

        assert_eq!(
            error,
            AppError::Validation {
                field: "email".to_string(),
                message: "该邮箱地址已经存在".to_string(),
            }
        );
    }

    #[test]
    fn rejects_invalid_mail_server_host() {
        let repository = InMemoryAccountRepository::default();
        let mut input = sample_add_account_input();
        input.imap.host = "bad host".to_string();

        let error = add_account(&repository, input).expect_err("非法主机名必须报错");

        assert_eq!(
            error,
            AppError::Validation {
                field: "imap.host".to_string(),
                message: "服务器地址格式不合法".to_string(),
            }
        );
    }

    #[test]
    fn forwards_sanitized_input_to_connection_tester() {
        let input = AccountConnectionTestInput {
            display_name: "  Work Outlook  ".to_string(),
            email: "  work@example.com  ".to_string(),
            login: "  work@example.com  ".to_string(),
            imap: MailServerConfig {
                host: "  imap.example.com  ".to_string(),
                port: 993,
                security: MailSecurity::Tls,
            },
            smtp: MailServerConfig {
                host: "  smtp.example.com  ".to_string(),
                port: 587,
                security: MailSecurity::StartTls,
            },
        };

        let result =
            test_account_connection(&PassingTester, input).expect("连接预检应成功返回结构化结果");

        assert_eq!(result.status, AccountConnectionStatus::Passed);
        assert_eq!(result.checks.len(), 1);
    }

    fn sample_add_account_input() -> AddAccountInput {
        AddAccountInput {
            display_name: "Primary Gmail".to_string(),
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
        }
    }
}
