use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceViewId {
    RecentVerification,
    AllInbox,
    SiteList,
    Accounts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageCategory {
    Registration,
    Security,
    Marketing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageStatus {
    Pending,
    Processed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceExtractKind {
    Code,
    Link,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavigationItem {
    pub id: WorkspaceViewId,
    pub label: String,
    pub badge: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceMessageItem {
    pub id: String,
    pub subject: String,
    pub sender: String,
    pub account_name: String,
    pub received_at: String,
    pub category: MessageCategory,
    pub status: MessageStatus,
    pub has_code: bool,
    pub has_link: bool,
    pub preview: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceMessageGroup {
    pub id: String,
    pub label: String,
    pub items: Vec<WorkspaceMessageItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceMessageDetail {
    pub id: String,
    pub subject: String,
    pub sender: String,
    pub account_name: String,
    pub received_at: String,
    pub category: MessageCategory,
    pub status: MessageStatus,
    pub site_hint: String,
    pub summary: String,
    pub extracted_code: Option<String>,
    pub verification_link: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceExtractItem {
    pub id: String,
    pub sender: String,
    pub kind: WorkspaceExtractKind,
    pub value: String,
    pub label: String,
    pub progress_percent: u8,
    pub expires_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceSiteSummary {
    pub id: String,
    pub label: String,
    pub hostname: String,
    pub pending_count: u32,
    pub latest_sender: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceBootstrapSnapshot {
    pub app_name: String,
    pub generated_at: String,
    pub default_view: WorkspaceViewId,
    pub navigation: Vec<NavigationItem>,
    pub message_groups: Vec<WorkspaceMessageGroup>,
    pub selected_message: WorkspaceMessageDetail,
    #[serde(default)]
    pub extracts: Vec<WorkspaceExtractItem>,
    #[serde(default)]
    pub site_summaries: Vec<WorkspaceSiteSummary>,
}
