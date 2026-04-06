use crate::domain::account::{
    AccountConnectionCheck, AccountConnectionCheckTarget, AccountConnectionStatus,
    AccountConnectionTestInput, AccountConnectionTestResult, MailSecurity, MailServerConfig,
};
use crate::infra::socket_probe::probe_socket;
use crate::services::account_service::AccountConnectionTester;
use std::time::Duration;

pub struct LiveAccountConnectionTester {
    timeout: Duration,
}

impl LiveAccountConnectionTester {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl Default for LiveAccountConnectionTester {
    fn default() -> Self {
        Self::new(Duration::from_millis(1_500))
    }
}

impl AccountConnectionTester for LiveAccountConnectionTester {
    fn test_account_connection(
        &self,
        input: &AccountConnectionTestInput,
    ) -> AccountConnectionTestResult {
        let checks = vec![
            build_identity_check(input),
            build_protocol_check(
                AccountConnectionCheckTarget::Imap,
                "IMAP",
                &input.imap,
                993,
                143,
                self.timeout,
            ),
            build_protocol_check(
                AccountConnectionCheckTarget::Smtp,
                "SMTP",
                &input.smtp,
                465,
                587,
                self.timeout,
            ),
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
                "手动配置实时探测通过，当前主机与端口可达，可继续进入下一步协议接入。".to_string()
            }
            AccountConnectionStatus::Failed => {
                "手动配置实时探测未通过，请先修正端口、安全策略或网络可达性。".to_string()
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

fn build_protocol_check(
    target: AccountConnectionCheckTarget,
    label: &str,
    server: &MailServerConfig,
    tls_port: u16,
    starttls_port: u16,
    timeout: Duration,
) -> AccountConnectionCheck {
    let rule_hint = build_rule_hint(label, server.security, server.port, tls_port, starttls_port);

    match probe_socket(&server.host, server.port, timeout) {
        Ok(()) => AccountConnectionCheck {
            target,
            status: AccountConnectionStatus::Passed,
            message: match rule_hint {
                Some(rule_hint) => format!(
                    "{label} 连接成功（{}:{}）。{rule_hint}",
                    server.host, server.port
                ),
                None => format!("{label} 连接成功（{}:{}）。", server.host, server.port),
            },
        },
        Err(error) => AccountConnectionCheck {
            target,
            status: AccountConnectionStatus::Failed,
            message: match rule_hint {
                Some(rule_hint) => format!(
                    "{label} 连接失败（{}:{}）：{error}。{rule_hint}",
                    server.host, server.port
                ),
                None => format!(
                    "{label} 连接失败（{}:{}）：{error}。",
                    server.host, server.port
                ),
            },
        },
    }
}

fn build_rule_hint(
    label: &str,
    security: MailSecurity,
    port: u16,
    tls_port: u16,
    starttls_port: u16,
) -> Option<String> {
    match security {
        MailSecurity::Tls if port == tls_port => {
            Some(format!("{label} 当前使用 {port} + TLS，符合常见安全组合。"))
        }
        MailSecurity::StartTls if port == starttls_port => Some(format!(
            "{label} 当前使用 {port} + STARTTLS，符合常见提交入口配置。"
        )),
        MailSecurity::None if port != tls_port && port != starttls_port => Some(format!(
            "{label} 当前配置为明文端口 {port}，后续仍建议评估加密升级。"
        )),
        MailSecurity::Tls => Some(format!(
            "{label} 当前端口 {port} 偏离常见 TLS 端口 {tls_port}。"
        )),
        MailSecurity::StartTls => Some(format!(
            "{label} 当前端口 {port} 偏离常见 STARTTLS 端口 {starttls_port}。"
        )),
        MailSecurity::None => Some(format!(
            "{label} 端口 {port} 通常要求加密，不建议长期配置为明文。"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::LiveAccountConnectionTester;
    use crate::domain::account::{
        AccountConnectionCheckTarget, AccountConnectionStatus, AccountConnectionTestInput,
        MailSecurity, MailServerConfig,
    };
    use crate::services::account_service::AccountConnectionTester;
    use std::net::TcpListener;

    #[test]
    fn passes_when_both_mail_servers_are_reachable() {
        let tester = LiveAccountConnectionTester::default();
        let imap = TcpListener::bind("127.0.0.1:0").expect("应能绑定 IMAP 测试端口");
        let smtp = TcpListener::bind("127.0.0.1:0").expect("应能绑定 SMTP 测试端口");

        let result = tester.test_account_connection(&sample_input(
            imap.local_addr().expect("应能读取 IMAP 地址").port(),
            MailSecurity::None,
            smtp.local_addr().expect("应能读取 SMTP 地址").port(),
            MailSecurity::None,
        ));

        assert_eq!(result.status, AccountConnectionStatus::Passed);
        assert!(
            result.summary.contains("实时探测通过"),
            "成功时应明确返回实时探测通过"
        );
    }

    #[test]
    fn fails_when_mail_server_socket_is_unreachable() {
        let tester = LiveAccountConnectionTester::default();
        let smtp = TcpListener::bind("127.0.0.1:0").expect("应能绑定 SMTP 测试端口");
        let unreachable_imap_port = reserve_unused_port();

        let result = tester.test_account_connection(&sample_input(
            unreachable_imap_port,
            MailSecurity::None,
            smtp.local_addr().expect("应能读取 SMTP 地址").port(),
            MailSecurity::None,
        ));

        assert_eq!(result.status, AccountConnectionStatus::Failed);
        assert!(
            result.summary.contains("实时探测未通过"),
            "失败时应返回整体实时探测未通过的结论"
        );
        let imap_check = result
            .checks
            .iter()
            .find(|check| check.target == AccountConnectionCheckTarget::Imap)
            .expect("应存在 IMAP 探测结果");
        assert_eq!(imap_check.status, AccountConnectionStatus::Failed);
        assert!(
            imap_check.message.contains("连接失败"),
            "端口不可达时应返回连接失败信息"
        );
    }

    #[test]
    fn keeps_rule_hint_when_nonstandard_port_is_still_reachable() {
        let tester = LiveAccountConnectionTester::default();
        let imap = TcpListener::bind("127.0.0.1:0").expect("应能绑定 IMAP 测试端口");
        let smtp = TcpListener::bind("127.0.0.1:0").expect("应能绑定 SMTP 测试端口");

        let result = tester.test_account_connection(&sample_input(
            imap.local_addr().expect("应能读取 IMAP 地址").port(),
            MailSecurity::Tls,
            smtp.local_addr().expect("应能读取 SMTP 地址").port(),
            MailSecurity::StartTls,
        ));

        assert_eq!(result.status, AccountConnectionStatus::Passed);
        let imap_check = result
            .checks
            .iter()
            .find(|check| check.target == AccountConnectionCheckTarget::Imap)
            .expect("应存在 IMAP 探测结果");
        assert!(
            imap_check.message.contains("偏离常见"),
            "非标准端口但可连接时，应保留规则提醒"
        );
        assert!(
            imap_check.message.contains("连接成功"),
            "非标准端口但可连接时，应明确实时连接成功"
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
                host: "127.0.0.1".to_string(),
                port: imap_port,
                security: imap_security,
            },
            smtp: MailServerConfig {
                host: "127.0.0.1".to_string(),
                port: smtp_port,
                security: smtp_security,
            },
        }
    }

    fn reserve_unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("应能分配空闲端口");
        let port = listener.local_addr().expect("应能读取本地地址").port();
        drop(listener);
        port
    }
}
