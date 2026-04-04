import { invoke } from "@tauri-apps/api/core";
import type {
  AccountConnectionTestInput,
  AccountConnectionTestResult,
  AccountSummary,
  AddAccountInput,
} from "../../features/accounts/model";

export function listAccounts() {
  return invoke<AccountSummary[]>("list_accounts");
}

export function addAccount(input: AddAccountInput) {
  return invoke<AccountSummary>("add_account", { input });
}

export function testAccountConnection(input: AccountConnectionTestInput) {
  return invoke<AccountConnectionTestResult>("test_account_connection", { input });
}
