export type WorkspaceViewId =
  | "recent_verification"
  | "all_inbox"
  | "site_list"
  | "accounts";

export type MessageCategory = "registration" | "security" | "marketing";
export type MessageStatus = "pending" | "processed";

export interface NavigationItem {
  id: WorkspaceViewId;
  label: string;
  badge: number;
}

export interface WorkspaceMessageItem {
  id: string;
  subject: string;
  sender: string;
  account_name: string;
  received_at: string;
  category: MessageCategory;
  status: MessageStatus;
  has_code: boolean;
  has_link: boolean;
  preview: string;
}

export interface WorkspaceMessageGroup {
  id: string;
  label: string;
  items: WorkspaceMessageItem[];
}

export interface WorkspaceMessageDetail {
  id: string;
  subject: string;
  sender: string;
  account_name: string;
  received_at: string;
  category: MessageCategory;
  status: MessageStatus;
  site_hint: string;
  summary: string;
  extracted_code: string | null;
  verification_link: string | null;
}

export interface WorkspaceBootstrapSnapshot {
  app_name: string;
  generated_at: string;
  default_view: WorkspaceViewId;
  navigation: NavigationItem[];
  message_groups: WorkspaceMessageGroup[];
  selected_message: WorkspaceMessageDetail;
}
