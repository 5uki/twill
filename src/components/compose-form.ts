import type {
  AccountSummary,
  ComposeMode,
  PreparedComposeDraft,
  SendMessageCommandInput,
  WorkspaceMessageDetail,
} from "../lib/app-types";

export interface ComposeFormDraft {
  accountId: string;
  to: string;
  subject: string;
  body: string;
}

export function createEmptyComposeFormDraft(
  accountId = "",
): ComposeFormDraft {
  return {
    accountId,
    to: "",
    subject: "",
    body: "",
  };
}

export function syncComposeDraftAccount(
  draft: ComposeFormDraft,
  accounts: AccountSummary[],
): ComposeFormDraft {
  if (accounts.length === 0) {
    return draft;
  }

  if (accounts.some((account) => account.id === draft.accountId)) {
    return draft;
  }

  return {
    ...draft,
    accountId: accounts[0].id,
  };
}

export function createNewComposeFormDraft(
  draft: ComposeFormDraft,
  accounts: AccountSummary[],
): ComposeFormDraft {
  return syncComposeDraftAccount(
    createEmptyComposeFormDraft(draft.accountId),
    accounts,
  );
}

export function buildSendMessageCommandInput(
  draft: ComposeFormDraft,
): SendMessageCommandInput {
  return {
    account_id: draft.accountId.trim(),
    to: draft.to.trim(),
    subject: draft.subject.trim(),
    body: draft.body.trim(),
  };
}

export function buildComposeFormDraft(
  draft: PreparedComposeDraft,
): ComposeFormDraft {
  return {
    accountId: draft.account_id,
    to: draft.to,
    subject: draft.subject,
    body: draft.body,
  };
}

export function prepareComposeDraftFromMessage(
  mode: ComposeMode,
  message: WorkspaceMessageDetail,
): PreparedComposeDraft {
  return {
    mode,
    account_id: message.account_id,
    to: mode === "reply" ? message.sender : "",
    subject: buildComposeSubject(mode, message.subject),
    body: buildComposeBody(mode, message),
    source_message_id: message.id,
  };
}

function buildComposeSubject(mode: ComposeMode, subject: string): string {
  if (mode === "new") {
    return subject.trim();
  }

  const prefix = mode === "reply" ? "Re:" : "Fwd:";
  const trimmedSubject = subject.trim();

  if (trimmedSubject.slice(0, prefix.length).toLowerCase() === prefix.toLowerCase()) {
    return trimmedSubject;
  }

  return `${prefix} ${trimmedSubject}`;
}

function buildComposeBody(
  mode: ComposeMode,
  message: WorkspaceMessageDetail,
): string {
  if (mode === "new") {
    return "";
  }

  const originalBody = message.body_text?.trim() || message.summary;

  if (mode === "reply") {
    const quotedBody = originalBody
      .split("\n")
      .map((line) => `> ${line}`)
      .join("\n");

    return `\n\n在 ${message.received_at}，${message.sender} 写道：\n${quotedBody}`;
  }

  return [
    "",
    "",
    "---------- 转发邮件 ----------",
    `发件人: ${message.sender}`,
    `账号: ${message.account_name}`,
    `时间: ${message.received_at}`,
    `主题: ${message.subject}`,
    "",
    originalBody,
  ].join("\n");
}
