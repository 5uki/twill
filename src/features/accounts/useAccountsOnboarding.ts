import { startTransition, useEffect, useState } from "react";
import { addAccount, listAccounts, testAccountConnection } from "../../lib/tauri/accounts";
import {
  createDefaultAccountFormState,
  toAddAccountInput,
  toConnectionTestInput,
  type AccountFormState,
} from "./form";
import type { AccountConnectionTestResult, AccountSummary } from "./model";

type AccountsState =
  | { status: "loading"; accounts: AccountSummary[] }
  | { status: "ready"; accounts: AccountSummary[] }
  | { status: "error"; accounts: AccountSummary[]; message: string };

function toErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback;
}

export function useAccountsOnboarding() {
  const [form, setForm] = useState<AccountFormState>(createDefaultAccountFormState);
  const [accountsState, setAccountsState] = useState<AccountsState>({
    status: "loading",
    accounts: [],
  });
  const [saveState, setSaveState] = useState<"idle" | "submitting">("idle");
  const [testState, setTestState] = useState<"idle" | "submitting">("idle");
  const [saveError, setSaveError] = useState<string | null>(null);
  const [testError, setTestError] = useState<string | null>(null);
  const [lastSavedAccount, setLastSavedAccount] = useState<AccountSummary | null>(null);
  const [lastTestResult, setLastTestResult] = useState<AccountConnectionTestResult | null>(null);

  async function loadAccounts() {
    startTransition(() => {
      setAccountsState((current) => ({
        status: "loading",
        accounts: current.accounts,
      }));
    });

    try {
      const accounts = await listAccounts();

      startTransition(() => {
        setAccountsState({
          status: "ready",
          accounts,
        });
      });
    } catch (error) {
      startTransition(() => {
        setAccountsState((current) => ({
          status: "error",
          accounts: current.accounts,
          message: toErrorMessage(error, "账户列表加载失败"),
        }));
      });
    }
  }

  useEffect(() => {
    void loadAccounts();
  }, []);

  function updateField(field: keyof AccountFormState, value: string) {
    setForm((current) => ({
      ...current,
      [field]: value,
    }));
  }

  async function saveAccountDraft() {
    setSaveState("submitting");
    setSaveError(null);

    try {
      const account = await addAccount(toAddAccountInput(form));

      startTransition(() => {
        setAccountsState((current) => ({
          status: "ready",
          accounts:
            current.status === "ready"
              ? [...current.accounts, account]
              : [account],
        }));
        setLastSavedAccount(account);
        setForm((current) => ({
          ...current,
          displayName: "",
          email: "",
          login: "",
        }));
      });
    } catch (error) {
      startTransition(() => {
        setSaveError(toErrorMessage(error, "账户保存失败"));
      });
    } finally {
      startTransition(() => {
        setSaveState("idle");
      });
    }
  }

  async function runConnectionTest() {
    setTestState("submitting");
    setTestError(null);

    try {
      const result = await testAccountConnection(toConnectionTestInput(form));

      startTransition(() => {
        setLastTestResult(result);
      });
    } catch (error) {
      startTransition(() => {
        setTestError(toErrorMessage(error, "账户连接预检失败"));
      });
    } finally {
      startTransition(() => {
        setTestState("idle");
      });
    }
  }

  return {
    form,
    accountsState,
    saveState,
    testState,
    saveError,
    testError,
    lastSavedAccount,
    lastTestResult,
    accountCount: accountsState.accounts.length,
    updateField,
    loadAccounts,
    saveAccountDraft,
    runConnectionTest,
  };
}
