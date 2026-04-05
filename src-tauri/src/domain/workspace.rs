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
pub enum MessageReadState {
    Unread,
    Read,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceMessageAction {
    CopyCode,
    OpenLink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceExtractKind {
    Code,
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceMailboxKind {
    Inbox,
    SpamJunk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceSyncState {
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceSyncPhase {
    First,
    Incremental,
}

fn default_message_read_state() -> MessageReadState {
    MessageReadState::Unread
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceSyncStatus {
    pub state: WorkspaceSyncState,
    pub summary: String,
    #[serde(default)]
    pub phase: Option<WorkspaceSyncPhase>,
    #[serde(default)]
    pub poll_interval_minutes: Option<u32>,
    #[serde(default)]
    pub retention_days: Option<u32>,
    #[serde(default)]
    pub next_poll_at: Option<String>,
    #[serde(default)]
    pub folders: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceMailboxSummary {
    pub id: String,
    #[serde(default)]
    pub account_id: String,
    pub account_name: String,
    pub label: String,
    pub kind: WorkspaceMailboxKind,
    pub total_count: u32,
    pub unread_count: u32,
    pub verification_count: u32,
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
    #[serde(default)]
    pub account_id: String,
    pub subject: String,
    pub sender: String,
    pub account_name: String,
    #[serde(default)]
    pub mailbox_id: String,
    #[serde(default)]
    pub mailbox_label: String,
    pub received_at: String,
    pub category: MessageCategory,
    pub status: MessageStatus,
    #[serde(default = "default_message_read_state")]
    pub read_state: MessageReadState,
    pub has_code: bool,
    pub has_link: bool,
    pub preview: String,
    #[serde(default)]
    pub prefetched_body: bool,
    #[serde(default)]
    pub synced_at: String,
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
    #[serde(default)]
    pub account_id: String,
    pub subject: String,
    pub sender: String,
    pub account_name: String,
    #[serde(default)]
    pub mailbox_id: String,
    #[serde(default)]
    pub mailbox_label: String,
    pub received_at: String,
    pub category: MessageCategory,
    pub status: MessageStatus,
    #[serde(default = "default_message_read_state")]
    pub read_state: MessageReadState,
    pub site_hint: String,
    pub summary: String,
    pub extracted_code: Option<String>,
    pub verification_link: Option<String>,
    #[serde(default)]
    pub original_message_url: Option<String>,
    #[serde(default)]
    pub body_text: Option<String>,
    #[serde(default)]
    pub prefetched_body: bool,
    #[serde(default)]
    pub synced_at: String,
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
pub struct WorkspaceSiteContextResolution {
    pub input: String,
    #[serde(default)]
    pub normalized_domain: Option<String>,
    #[serde(default)]
    pub matched_site: Option<WorkspaceSiteSummary>,
    #[serde(default)]
    pub candidate_sites: Vec<WorkspaceSiteSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceMessageActionResult {
    pub action: WorkspaceMessageAction,
    pub message_id: String,
    #[serde(default)]
    pub copied_value: Option<String>,
    #[serde(default)]
    pub opened_url: Option<String>,
    pub snapshot: WorkspaceBootstrapSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceMessageOpenResult {
    pub detail: WorkspaceMessageDetail,
    pub snapshot: WorkspaceBootstrapSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceMessageOriginalOpenResult {
    pub message_id: String,
    pub original_url: String,
    pub snapshot: WorkspaceBootstrapSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceBootstrapSnapshot {
    pub app_name: String,
    pub generated_at: String,
    pub default_view: WorkspaceViewId,
    pub navigation: Vec<NavigationItem>,
    #[serde(default)]
    pub mailboxes: Vec<WorkspaceMailboxSummary>,
    pub message_groups: Vec<WorkspaceMessageGroup>,
    pub selected_message: WorkspaceMessageDetail,
    #[serde(default)]
    pub message_details: Vec<WorkspaceMessageDetail>,
    #[serde(default)]
    pub extracts: Vec<WorkspaceExtractItem>,
    #[serde(default)]
    pub site_summaries: Vec<WorkspaceSiteSummary>,
    #[serde(default)]
    pub sync_status: Option<WorkspaceSyncStatus>,
}
