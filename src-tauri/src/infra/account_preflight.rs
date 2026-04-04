use crate::domain::account::{
    AccountConnectionCheck, AccountConnectionCheckTarget, AccountConnectionStatus,
    AccountConnectionTestInput, AccountConnectionTestResult, MailSecurity,
};
use crate::services::account_service::AccountConnectionTester;

pub struct RuleBasedAccountConnectionTester;

impl AccountConnectionTester for RuleBasedAccountConnectionTester {
    fn test_account_connection(
        &self,
        input: &AccountConnectionTestInput,
    ) -> AccountConnectionTestResult {
        let checks = vec![
            build_identity_check(input),
            build_imap_check(&input.imap),
            build_smtp_check(&input.smtp),
        ];

        let status = if checks
            .iter()
            .all(|check| check.status == AccountConnectionStatus::Passed)
        {
            AccountConnectionStatus::Passed
        } else {
            AccountConnectionStatus::Failed
        };

        let summary = match status {
            AccountConnectionStatus::Passed => {
                "手动配置连接预检通过，可以进入后续安全存储与真实协议接入。".to_string()
            }
            AccountConnectionStatus::Failed => {
                "手动配置连接预检未通过，请先修正服务器端口或安全策略。".to_string()
            }
        };

        AccountConnectionTestResult {
            status,
            summary,
            checks,
        }
    }
}

fn build_identity_check(input: &AccountConnectionTestInput) -> AccountConnectionCheck {
    AccountConnectionCheck {
        target: AccountConnectionCheckTarget::Identity,
        status: AccountConnectionStatus::Passed,
        message: format!(
            "邮箱 {} 与登录名 {} 已通过基础格式校验。",
            input.email, input.login
        ),
    }
}

fn build_imap_check(server: &crate::domain::account::MailServerConfig) -> AccountConnectionCheck {
    build_protocol_check(
        AccountConnectionCheckTarget::Imap,
        "IMAP",
        server.security,
        server.port,
        993,
        143,
    )
}

fn build_smtp_check(server: &crate::domain::account::MailServerConfig) -> AccountConnectionCheck {
    build_protocol_check(
        AccountConnectionCheckTarget::Smtp,
        "SMTP",
        server.security,
        server.port,
        465,
        587,
    )
}

fn build_protocol_check(
    target: AccountConnectionCheckTarget,
    label: &str,
    security: MailSecurity,
    port: u16,
    tls_port: u16,
    starttls_port: u16,
) -> AccountConnectionCheck {
    match security {
        MailSecurity::Tls if port == tls_port => AccountConnectionCheck {
            target,
            status: AccountConnectionStatus::Passed,
            message: format!("{label} 使用 {port} + TLS，是常见的安全组合。"),
        },
        MailSecurity::StartTls if port == starttls_port => AccountConnectionCheck {
            target,
            status: AccountConnectionStatus::Passed,
            message: format!("{label} 使用 {port} + STARTTLS，适合作为提交入口。"),
        },
        MailSecurity::None if port != tls_port && port != starttls_port => AccountConnectionCheck {
            target,
            status: AccountConnectionStatus::Passed,
            message: format!("{label} 当前配置为明文端口 {port}，后续应按服务商能力评估加密升级。"),
        },
        MailSecurity::Tls => AccountConnectionCheck {
            target,
            status: AccountConnectionStatus::Failed,
            message: format!(
                "{label} 选择 TLS 时更常见的端口是 {tls_port}，当前 {port} 组合可疑。"
            ),
        },
        MailSecurity::StartTls => AccountConnectionCheck {
            target,
            status: AccountConnectionStatus::Failed,
            message: format!(
                "{label} 选择 STARTTLS 时更常见的端口是 {starttls_port}，当前 {port} 组合可疑。"
            ),
        },
        MailSecurity::None => AccountConnectionCheck {
            target,
            status: AccountConnectionStatus::Failed,
            message: format!("{label} 端口 {port} 通常要求加密，不建议配置为明文。"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::RuleBasedAccountConnectionTester;
    use crate::domain::account::{
        AccountConnectionStatus, AccountConnectionTestInput, MailSecurity, MailServerConfig,
    };
    use crate::services::account_service::AccountConnectionTester;

    #[test]
    fn passes_for_common_tls_and_starttls_combination() {
        let tester = RuleBasedAccountConnectionTester;

        let result = tester.test_account_connection(&sample_input(
            993,
            MailSecurity::Tls,
            587,
            MailSecurity::StartTls,
        ));

        assert_eq!(result.status, AccountConnectionStatus::Passed);
    }

    #[test]
    fn fails_for_mismatched_tls_port() {
        let tester = RuleBasedAccountConnectionTester;

        let result = tester.test_account_connection(&sample_input(
            143,
            MailSecurity::Tls,
            587,
            MailSecurity::StartTls,
        ));

        assert_eq!(result.status, AccountConnectionStatus::Failed);
        assert!(
            result.summary.contains("未通过"),
            "失败时应返回整体未通过的结论"
        );
    }

    fn sample_input(
        imap_port: u16,
        imap_security: MailSecurity,
        smtp_port: u16,
        smtp_security: MailSecurity,
    ) -> AccountConnectionTestInput {
        AccountConnectionTestInput {
            display_name: "Primary".to_string(),
            email: "primary@example.com".to_string(),
            login: "primary@example.com".to_string(),
            imap: MailServerConfig {
                host: "imap.example.com".to_string(),
                port: imap_port,
                security: imap_security,
            },
            smtp: MailServerConfig {
                host: "smtp.example.com".to_string(),
                port: smtp_port,
                security: smtp_security,
            },
        }
    }
}
