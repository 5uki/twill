use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MailSecurity {
    None,
    StartTls,
    Tls,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MailServerConfig {
    pub host: String,
    pub port: u16,
    pub security: MailSecurity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddAccountInput {
    pub display_name: String,
    pub email: String,
    pub login: String,
    pub password: String,
    pub imap: MailServerConfig,
    pub smtp: MailServerConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountConnectionTestInput {
    pub display_name: String,
    pub email: String,
    pub login: String,
    pub imap: MailServerConfig,
    pub smtp: MailServerConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountSummary {
    pub id: String,
    pub display_name: String,
    pub email: String,
    pub login: String,
    #[serde(default)]
    pub credential_state: AccountCredentialState,
    pub imap: MailServerConfig,
    pub smtp: MailServerConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AccountCredentialState {
    #[default]
    Missing,
    Stored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountConnectionStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountConnectionCheckTarget {
    Identity,
    Imap,
    Smtp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountConnectionCheck {
    pub target: AccountConnectionCheckTarget,
    pub status: AccountConnectionStatus,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountConnectionTestResult {
    pub status: AccountConnectionStatus,
    pub summary: String,
    pub checks: Vec<AccountConnectionCheck>,
}
