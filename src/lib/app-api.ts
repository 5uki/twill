import { invoke, isTauri } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import workspaceBootstrap from "../data/workspace-bootstrap.json";
import type {
  AccountConnectionCommandInput,
  AccountConnectionTestResult,
  AccountSummary,
  AddAccountCommandInput,
  WorkspaceBootstrapSnapshot,
} from "./app-types";

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
