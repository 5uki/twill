import { invoke, isTauri } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import workspaceBootstrap from "../data/workspace-bootstrap.json";
import type {
  AccountConnectionCommandInput,
  AccountConnectionTestResult,
  AccountSummary,
  MessageStatus,
  AddAccountCommandInput,
  WorkspaceBootstrapSnapshot,
  WorkspaceMessageAction,
  WorkspaceMessageActionResult,
  WorkspaceMessageDetail,
  WorkspaceMessageFilter,
  WorkspaceMessageItem,
  WorkspaceMessageOpenResult,
  WorkspaceMessageOriginalOpenResult,
  MessageReadState,
  WorkspaceSiteContextResolution,
} from "./app-types";
import {
  applyWorkspaceMessageAction as applyWorkspaceMessageActionToSnapshot,
  applyWorkspaceMessageReadState,
  confirmWorkspaceSite as confirmWorkspaceSiteInSnapshot,
  openWorkspaceMessage as openWorkspaceMessageInSnapshot,
  openWorkspaceMessageOriginal as openWorkspaceMessageOriginalInSnapshot,
  applyWorkspaceMessageStatus,
  filterWorkspaceMessages,
  readWorkspaceMessageFromSnapshot,
  resolveWorkspaceSiteContext as resolveWorkspaceSiteContextFromSnapshot,
} from "./workspace-reading";

const fallbackWorkspaceBootstrap =
  workspaceBootstrap as WorkspaceBootstrapSnapshot;

export function hasDesktopRuntime(): boolean {
  return isTauri();
}

export async function loadWorkspaceBootstrap(): Promise<WorkspaceBootstrapSnapshot> {
  if (!hasDesktopRuntime()) {
    return fallbackWorkspaceBootstrap;
  }

  try {
    return await invoke<WorkspaceBootstrapSnapshot>("load_workspace_bootstrap");
  } catch {
    return fallbackWorkspaceBootstrap;
  }
}

export async function syncWorkspace(): Promise<WorkspaceBootstrapSnapshot> {
  if (!hasDesktopRuntime()) {
    return fallbackWorkspaceBootstrap;
  }

  return invoke<WorkspaceBootstrapSnapshot>("sync_workspace");
}

export async function listWorkspaceMessages(
  filter: WorkspaceMessageFilter = {},
): Promise<WorkspaceMessageItem[]> {
  if (!hasDesktopRuntime()) {
    return filterWorkspaceMessages(fallbackWorkspaceBootstrap, filter);
  }

  return invoke<WorkspaceMessageItem[]>("list_workspace_messages", { filter });
}

export async function readWorkspaceMessage(
  messageId: string,
): Promise<WorkspaceMessageDetail> {
  if (!hasDesktopRuntime()) {
    return readWorkspaceMessageFromSnapshot(fallbackWorkspaceBootstrap, messageId);
  }

  return invoke<WorkspaceMessageDetail>("read_workspace_message", { messageId });
}

export async function openWorkspaceMessage(
  messageId: string,
): Promise<WorkspaceMessageOpenResult> {
  if (!hasDesktopRuntime()) {
    return openWorkspaceMessageInSnapshot(fallbackWorkspaceBootstrap, messageId);
  }

  return invoke<WorkspaceMessageOpenResult>("open_workspace_message", { messageId });
}

export async function updateWorkspaceMessageStatus(
  messageId: string,
  status: MessageStatus,
): Promise<WorkspaceBootstrapSnapshot> {
  if (!hasDesktopRuntime()) {
    return applyWorkspaceMessageStatus(fallbackWorkspaceBootstrap, messageId, status);
  }

  return invoke<WorkspaceBootstrapSnapshot>("update_workspace_message_status", {
    messageId,
    status,
  });
}

export async function updateWorkspaceMessageReadState(
  messageId: string,
  readState: MessageReadState,
): Promise<WorkspaceBootstrapSnapshot> {
  if (!hasDesktopRuntime()) {
    return applyWorkspaceMessageReadState(
      fallbackWorkspaceBootstrap,
      messageId,
      readState,
    );
  }

  return invoke<WorkspaceBootstrapSnapshot>("update_workspace_message_read_state", {
    messageId,
    readState,
  });
}

export async function applyWorkspaceMessageAction(
  messageId: string,
  action: WorkspaceMessageAction,
): Promise<WorkspaceMessageActionResult> {
  if (!hasDesktopRuntime()) {
    return applyWorkspaceMessageActionToSnapshot(
      fallbackWorkspaceBootstrap,
      messageId,
      action,
    );
  }

  return invoke<WorkspaceMessageActionResult>("apply_workspace_message_action", {
    messageId,
    action,
  });
}

export async function openWorkspaceMessageOriginal(
  messageId: string,
): Promise<WorkspaceMessageOriginalOpenResult> {
  if (!hasDesktopRuntime()) {
    return openWorkspaceMessageOriginalInSnapshot(
      fallbackWorkspaceBootstrap,
      messageId,
    );
  }

  return invoke<WorkspaceMessageOriginalOpenResult>(
    "open_workspace_message_original",
    { messageId },
  );
}

export async function resolveWorkspaceSiteContext(
  input: string,
): Promise<WorkspaceSiteContextResolution> {
  if (!hasDesktopRuntime()) {
    return resolveWorkspaceSiteContextFromSnapshot(fallbackWorkspaceBootstrap, input);
  }

  return invoke<WorkspaceSiteContextResolution>("resolve_workspace_site_context", {
    input,
  });
}

export async function confirmWorkspaceSite(
  input: string,
  label?: string | null,
): Promise<WorkspaceBootstrapSnapshot> {
  if (!hasDesktopRuntime()) {
    return confirmWorkspaceSiteInSnapshot(fallbackWorkspaceBootstrap, input, label);
  }

  return invoke<WorkspaceBootstrapSnapshot>("confirm_workspace_site", {
    input,
    label,
  });
}

export async function listAccounts(): Promise<AccountSummary[]> {
  if (!hasDesktopRuntime()) {
    return [];
  }

  return invoke<AccountSummary[]>("list_accounts");
}

export async function addAccount(
  input: AddAccountCommandInput,
): Promise<AccountSummary> {
  return invoke<AccountSummary>("add_account", { input });
}

export async function testAccountConnection(
  input: AccountConnectionCommandInput,
): Promise<AccountConnectionTestResult> {
  return invoke<AccountConnectionTestResult>("test_account_connection", {
    input,
  });
}

export async function openExternalUrl(url: string): Promise<void> {
  if (hasDesktopRuntime()) {
    await openUrl(url);
    return;
  }

  window.open(url, "_blank", "noopener,noreferrer");
}

export function getErrorMessage(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }

  if (
    typeof error === "object" &&
    error !== null &&
    "message" in error &&
    typeof error.message === "string"
  ) {
    return error.message;
  }

  return "请求失败，请稍后重试。";
}
