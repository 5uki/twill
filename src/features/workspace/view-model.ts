import type {
  MessageCategory,
  MessageStatus,
  WorkspaceBootstrapSnapshot,
  WorkspaceMessageDetail,
  WorkspaceMessageItem,
  WorkspaceViewId,
} from "./model";

export interface WorkspaceActionViewModel {
  id: "copy_code" | "open_link" | "open_message";
  label: string;
  appearance: "primary" | "secondary";
  body: string;
  disabled: boolean;
}

export interface WorkspaceViewModel {
  appName: string;
  title: string;
  subtitle: string;
  generatedAtLabel: string;
  navigation: Array<{
    id: WorkspaceViewId;
    label: string;
    badge: number;
    isActive: boolean;
  }>;
  groups: Array<{
    id: string;
    label: string;
    count: number;
    items: Array<{
      id: string;
      subject: string;
      sender: string;
      accountName: string;
      receivedAt: string;
      status: MessageStatus;
      categoryLabel: string;
      preview: string;
      flags: string[];
      isSelected: boolean;
    }>;
  }>;
  detail: {
    subject: string;
    sender: string;
    accountName: string;
    receivedAt: string;
    siteHint: string;
    categoryLabel: string;
    statusLabel: string;
    summary: string;
    extractedCode: string | null;
    verificationLink: string | null;
    actions: WorkspaceActionViewModel[];
  };
}

const VIEW_TITLES: Record<WorkspaceViewId, string> = {
  recent_verification: "Recent verification",
  all_inbox: "All inbox",
  site_list: "Sites",
  accounts: "Accounts",
};

const CATEGORY_TITLES: Record<MessageCategory, string> = {
  registration: "注册",
  security: "安全",
  marketing: "营销",
};

const STATUS_TITLES: Record<MessageStatus, string> = {
  pending: "待处理",
  processed: "已处理",
};

export function toWorkspaceViewModel(
  snapshot: WorkspaceBootstrapSnapshot,
): WorkspaceViewModel {
  return {
    appName: snapshot.app_name,
    title: VIEW_TITLES[snapshot.default_view],
    subtitle: "多账号验证邮件工作台 · M0 壳层 + M1 账户接入首切片",
    generatedAtLabel: formatTimestamp(snapshot.generated_at),
    navigation: snapshot.navigation.map((item) => ({
      ...item,
      isActive: item.id === snapshot.default_view,
    })),
    groups: snapshot.message_groups.map((group) => ({
      id: group.id,
      label: group.label,
      count: group.items.length,
      items: group.items.map((item) => toMessageItemViewModel(item, snapshot.selected_message.id)),
    })),
    detail: toDetailViewModel(snapshot.selected_message),
  };
}

function toMessageItemViewModel(item: WorkspaceMessageItem, selectedId: string) {
  const flags = [];

  if (item.has_code) {
    flags.push("验证码");
  }

  if (item.has_link) {
    flags.push("验证链接");
  }

  return {
    id: item.id,
    subject: item.subject,
    sender: item.sender,
    accountName: item.account_name,
    receivedAt: formatTimestamp(item.received_at),
    status: item.status,
    categoryLabel: CATEGORY_TITLES[item.category],
    preview: item.preview,
    flags,
    isSelected: item.id === selectedId,
  };
}

function toDetailViewModel(detail: WorkspaceMessageDetail) {
  const actions: WorkspaceActionViewModel[] = [
    {
      id: "copy_code",
      label: "复制验证码",
      appearance: "primary",
      body: detail.extracted_code ?? "当前邮件没有可直接复制的验证码",
      disabled: detail.extracted_code === null,
    },
    {
      id: "open_link",
      label: "打开验证链接",
      appearance: "secondary",
      body: detail.verification_link ?? "当前邮件没有验证链接",
      disabled: detail.verification_link === null,
    },
    {
      id: "open_message",
      label: "查看原邮件",
      appearance: "secondary",
      body: `${detail.account_name} · ${detail.sender}`,
      disabled: false,
    },
  ];

  return {
    subject: detail.subject,
    sender: detail.sender,
    accountName: detail.account_name,
    receivedAt: formatTimestamp(detail.received_at),
    siteHint: detail.site_hint,
    categoryLabel: CATEGORY_TITLES[detail.category],
    statusLabel: STATUS_TITLES[detail.status],
    summary: detail.summary,
    extractedCode: detail.extracted_code,
    verificationLink: detail.verification_link,
    actions,
  };
}

function formatTimestamp(value: string) {
  return value.replace("T", " ").replace("Z", " UTC");
}
