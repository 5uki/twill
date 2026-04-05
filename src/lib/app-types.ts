export type MailSecurity = "none" | "start_tls" | "tls";
export type AccountCredentialState = "missing" | "stored";
export type AccountConnectionStatus = "passed" | "failed";
export type AccountConnectionCheckTarget = "identity" | "imap" | "smtp";
export type WorkspaceViewId =
  | "recent_verification"
  | "all_inbox"
  | "site_list"
  | "accounts";
export type MessageCategory = "registration" | "security" | "marketing";
export type MessageStatus = "pending" | "processed";
export type WorkspaceExtractKind = "code" | "link";
export type WorkspaceSyncState = "ready";
export type WorkspaceSyncPhase = "first" | "incremental";
export type WorkspaceMailboxKind = "inbox" | "spam_junk";

export interface MailServerConfig {
  host: string;
  port: number;
  security: MailSecurity;
}

export interface AccountSummary {
  id: string;
  display_name: string;
  email: string;
  login: string;
  credential_state: AccountCredentialState;
  imap: MailServerConfig;
  smtp: MailServerConfig;
}

export interface AccountConnectionCheck {
  target: AccountConnectionCheckTarget;
  status: AccountConnectionStatus;
  message: string;
}

export interface AccountConnectionTestResult {
  status: AccountConnectionStatus;
  summary: string;
  checks: AccountConnectionCheck[];
}

export interface NavigationItem {
  id: WorkspaceViewId;
  label: string;
  badge: number;
}

export interface WorkspaceMessageItem {
  id: string;
  account_id: string;
  subject: string;
  sender: string;
  account_name: string;
  mailbox_id: string;
  mailbox_label: string;
  received_at: string;
  category: MessageCategory;
  status: MessageStatus;
  has_code: boolean;
  has_link: boolean;
  preview: string;
  prefetched_body: boolean;
  synced_at: string;
}

export interface WorkspaceMessageGroup {
  id: string;
  label: string;
  items: WorkspaceMessageItem[];
}

export interface WorkspaceMessageDetail {
  id: string;
  account_id: string;
  subject: string;
  sender: string;
  account_name: string;
  mailbox_id: string;
  mailbox_label: string;
  received_at: string;
  category: MessageCategory;
  status: MessageStatus;
  site_hint: string;
  summary: string;
  extracted_code: string | null;
  verification_link: string | null;
  body_text?: string | null;
  prefetched_body: boolean;
  synced_at: string;
}

export interface WorkspaceExtractItem {
  id: string;
  sender: string;
  kind: WorkspaceExtractKind;
  value: string;
  label: string;
  progress_percent: number;
  expires_label: string;
}

export interface WorkspaceSiteSummary {
  id: string;
  label: string;
  hostname: string;
  pending_count: number;
  latest_sender: string;
}

export interface WorkspaceSyncStatus {
  state: WorkspaceSyncState;
  summary: string;
  phase?: WorkspaceSyncPhase | null;
  poll_interval_minutes?: number | null;
  retention_days?: number | null;
  next_poll_at?: string | null;
  folders?: string[];
}

export interface WorkspaceMailboxSummary {
  id: string;
  account_id: string;
  account_name: string;
  label: string;
  kind: WorkspaceMailboxKind;
  total_count: number;
  unread_count: number;
  verification_count: number;
}

export interface WorkspaceBootstrapSnapshot {
  app_name: string;
  generated_at: string;
  default_view: WorkspaceViewId;
  navigation: NavigationItem[];
  mailboxes: WorkspaceMailboxSummary[];
  message_groups: WorkspaceMessageGroup[];
  selected_message: WorkspaceMessageDetail;
  message_details: WorkspaceMessageDetail[];
  extracts: WorkspaceExtractItem[];
  site_summaries: WorkspaceSiteSummary[];
  sync_status?: WorkspaceSyncStatus | null;
}

export interface AddAccountCommandInput {
  display_name: string;
  email: string;
  login: string;
  password: string;
  imap: MailServerConfig;
  smtp: MailServerConfig;
}

export interface AccountConnectionCommandInput {
  display_name: string;
  email: string;
  login: string;
  imap: MailServerConfig;
  smtp: MailServerConfig;
}
