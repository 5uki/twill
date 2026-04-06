use crate::domain::compose::MessageDeliveryMode;
use crate::domain::error::AppError;
use crate::infra::socket_probe::probe_socket;
use crate::services::compose_service::{
    MessageDeliveryClient, MessageDeliveryReceipt, MessageDeliveryRequest,
};
use std::time::Duration;

pub struct LiveComposeDeliveryClient {
    timeout: Duration,
}

impl LiveComposeDeliveryClient {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl Default for LiveComposeDeliveryClient {
    fn default() -> Self {
        Self::new(Duration::from_millis(1_500))
    }
}

impl MessageDeliveryClient for LiveComposeDeliveryClient {
    fn send_message(
        &self,
        request: &MessageDeliveryRequest,
    ) -> Result<MessageDeliveryReceipt, AppError> {
        probe_socket(&request.smtp.host, request.smtp.port, self.timeout).map_err(|error| {
            AppError::Validation {
                field: "smtp".to_string(),
                message: format!(
                    "SMTP 提交通道不可达（{}:{}）：{error}",
                    request.smtp.host, request.smtp.port
                ),
            }
        })?;

        Ok(MessageDeliveryReceipt {
            delivery_mode: MessageDeliveryMode::Simulated,
            summary: format!(
                "已验证 {} 的 SMTP 提交通道可达，并生成模拟发送回执。",
                request.account_id
            ),
            smtp_endpoint: format!("{}:{}", request.smtp.host, request.smtp.port),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::LiveComposeDeliveryClient;
    use crate::domain::account::{MailSecurity, MailServerConfig};
    use crate::domain::compose::MessageDeliveryMode;
    use crate::domain::error::AppError;
    use crate::services::compose_service::{MessageDeliveryClient, MessageDeliveryRequest};
    use std::net::TcpListener;

    #[test]
    fn returns_simulated_receipt_when_smtp_socket_is_reachable() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("应能绑定 SMTP 测试端口");
        let client = LiveComposeDeliveryClient::default();

        let result = client
            .send_message(&MessageDeliveryRequest {
                account_id: "acct_primary-example-com".to_string(),
                login: "primary@example.com".to_string(),
                password: "app-password".to_string(),
                smtp: MailServerConfig {
                    host: "127.0.0.1".to_string(),
                    port: listener.local_addr().expect("应能读取本地地址").port(),
                    security: MailSecurity::None,
                },
                to: "dev@example.com".to_string(),
                subject: "Launch".to_string(),
                body: "Ready".to_string(),
            })
            .expect("SMTP 可达时应返回模拟发送回执");

        assert_eq!(result.delivery_mode, MessageDeliveryMode::Simulated);
        assert!(result.summary.contains("模拟发送回执"));
    }

    #[test]
    fn returns_validation_error_when_smtp_socket_is_unreachable() {
        let client = LiveComposeDeliveryClient::default();
        let port = reserve_unused_port();

        let error = client
            .send_message(&MessageDeliveryRequest {
                account_id: "acct_primary-example-com".to_string(),
                login: "primary@example.com".to_string(),
                password: "app-password".to_string(),
                smtp: MailServerConfig {
                    host: "127.0.0.1".to_string(),
                    port,
                    security: MailSecurity::None,
                },
                to: "dev@example.com".to_string(),
                subject: "Launch".to_string(),
                body: "Ready".to_string(),
            })
            .expect_err("SMTP 不可达时必须返回错误");

        match error {
            AppError::Validation { field, message } => {
                assert_eq!(field, "smtp");
                assert!(message.contains("SMTP 提交通道不可达"));
                assert!(message.contains(&port.to_string()));
            }
            other => panic!("应返回 smtp 校验错误，实际得到: {other:?}"),
        }
    }

    fn reserve_unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("应能分配空闲端口");
        let port = listener.local_addr().expect("应能读取本地地址").port();
        drop(listener);
        port
    }
}
