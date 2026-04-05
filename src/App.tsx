import { FluentProvider } from "@fluentui/react-components";
import {
  MailInboxRegular,
  PersonAccountsRegular,
  ServerRegular,
  ShieldKeyholeRegular,
} from "@fluentui/react-icons";
import { startTransition, useEffect, useState } from "react";
import "./App.css";
import { AccountsWorkspace } from "./components/AccountsWorkspace";
import {
  buildAccountCommandInput,
  buildAccountConnectionCommandInput,
  createEmptyAccountFormDraft,
  type AccountFormDraft,
} from "./components/account-form";
import { MailWorkspace } from "./components/MailWorkspace";
import {
  Sidebar,
  type SidebarGroup,
  type SidebarItem,
} from "./components/Sidebar";
import { Titlebar } from "./components/Titlebar";
import { TopHeader } from "./components/TopHeader";
import {
  addAccount,
  getErrorMessage,
  hasDesktopRuntime,
  listAccounts,
  loadWorkspaceBootstrap,
  openExternalUrl,
  syncWorkspace,
  testAccountConnection,
} from "./lib/app-api";
import type {
  AccountConnectionTestResult,
  AccountSummary,
  WorkspaceBootstrapSnapshot,
  WorkspaceViewId,
} from "./lib/app-types";
import {
  getWorkspaceGroup,
  getWorkspaceGroupLabel,
  getWorkspaceNavigationLabel,
  mapWorkspaceViewToCategory,
  type WorkspaceCategory,
} from "./lib/workspace-view";
import { twillTheme } from "./theme";
import workspaceBootstrap from "./data/workspace-bootstrap.json";

const fallbackWorkspaceSnapshot =
  workspaceBootstrap as WorkspaceBootstrapSnapshot;

const navigationIconMap: Record<WorkspaceViewId, JSX.Element> = {
  recent_verification: <ShieldKeyholeRegular />,
  all_inbox: <MailInboxRegular />,
  site_list: <ServerRegular />,
  accounts: <PersonAccountsRegular />,
};

function App() {
  const runtimeAvailable = hasDesktopRuntime();
  const [workspaceSnapshot, setWorkspaceSnapshot] =
    useState<WorkspaceBootstrapSnapshot>(fallbackWorkspaceSnapshot);
  const [activeCategory, setActiveCategory] = useState<WorkspaceCategory>(
    mapWorkspaceViewToCategory(fallbackWorkspaceSnapshot.default_view),
  );
  const [accounts, setAccounts] = useState<AccountSummary[]>([]);
  const [accountDraft, setAccountDraft] = useState<AccountFormDraft>(
    createEmptyAccountFormDraft(),
  );
  const [accountError, setAccountError] = useState<string | null>(null);
  const [probeResult, setProbeResult] =
    useState<AccountConnectionTestResult | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [isTesting, setIsTesting] = useState(false);
  const [isSyncing, setIsSyncing] = useState(false);
  const [syncErrorMessage, setSyncErrorMessage] = useState<string | null>(null);

  const resolveWorkspaceSnapshot = async (nextAccounts: AccountSummary[]) => {
    if (nextAccounts.length === 0) {
      return {
        snapshot: await loadWorkspaceBootstrap(),
        errorMessage: null,
      };
    }

    try {
      return {
        snapshot: await syncWorkspace(),
        errorMessage: null,
      };
    } catch {
      return {
        snapshot: await loadWorkspaceBootstrap(),
        errorMessage: "暂时无法刷新收件箱，已显示当前快照。",
      };
    }
  };

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      const nextAccounts = await listAccounts();

      if (!cancelled) {
        setIsSyncing(nextAccounts.length > 0);
      }

      const workspaceResult = await resolveWorkspaceSnapshot(nextAccounts);

      if (cancelled) {
        return;
      }

      setIsSyncing(false);

      startTransition(() => {
        setWorkspaceSnapshot(workspaceResult.snapshot);
        setActiveCategory(
          mapWorkspaceViewToCategory(workspaceResult.snapshot.default_view),
        );
        setAccounts(nextAccounts);
        setSyncErrorMessage(workspaceResult.errorMessage);
      });
    };

    void load();

    return () => {
      cancelled = true;
    };
  }, []);

  const sidebarItems: SidebarItem[] = workspaceSnapshot.navigation.map((item) => {
    const category = mapWorkspaceViewToCategory(item.id);

    return {
      id: category,
      label: getWorkspaceNavigationLabel(category),
      badge: item.badge,
      icon: navigationIconMap[item.id],
    };
  });

  const sidebarGroups: SidebarGroup[] = (["mail", "manage"] as const).map(
    (group) => ({
      id: group,
      label: getWorkspaceGroupLabel(group),
      items: sidebarItems.filter((item) => getWorkspaceGroup(item.id) === group),
    }),
  );

  const updateDraft = (field: keyof AccountFormDraft, value: string) => {
    setAccountDraft((current) => ({
      ...current,
      [field]: value,
    }));
  };

  const refreshAccounts = async () => {
    try {
      const nextAccounts = await listAccounts();

      startTransition(() => {
        setAccounts(nextAccounts);
      });
    } catch (error) {
      setAccountError(getErrorMessage(error));
    }
  };

  const handleSaveAccount = async () => {
    try {
      setAccountError(null);
      setIsSaving(true);
      await addAccount(buildAccountCommandInput(accountDraft));
      const nextAccounts = await listAccounts();
      setIsSyncing(nextAccounts.length > 0);
      const workspaceResult = await resolveWorkspaceSnapshot(nextAccounts);

      startTransition(() => {
        setAccounts(nextAccounts);
        setWorkspaceSnapshot(workspaceResult.snapshot);
        setAccountDraft(createEmptyAccountFormDraft());
        setProbeResult(null);
        setSyncErrorMessage(workspaceResult.errorMessage);
      });
    } catch (error) {
      setAccountError(getErrorMessage(error));
    } finally {
      setIsSaving(false);
      setIsSyncing(false);
    }
  };

  const handleTestAccount = async () => {
    try {
      setAccountError(null);
      setIsTesting(true);
      const nextProbeResult = await testAccountConnection(
        buildAccountConnectionCommandInput(accountDraft),
      );

      startTransition(() => {
        setProbeResult(nextProbeResult);
      });
    } catch (error) {
      setAccountError(getErrorMessage(error));
      setProbeResult(null);
    } finally {
      setIsTesting(false);
    }
  };

  const handleSyncWorkspace = async () => {
    try {
      setSyncErrorMessage(null);
      setIsSyncing(true);
      const nextAccounts = await listAccounts();
      const workspaceResult = await resolveWorkspaceSnapshot(nextAccounts);

      startTransition(() => {
        setAccounts(nextAccounts);
        setWorkspaceSnapshot(workspaceResult.snapshot);
        setSyncErrorMessage(workspaceResult.errorMessage);
      });
    } finally {
      setIsSyncing(false);
    }
  };

  const syncSummary =
    syncErrorMessage ??
    workspaceSnapshot.sync_status?.summary ??
    (runtimeAvailable ? "添加账号后可同步最近邮件" : "桌面端可同步最新收件箱");

  return (
    <FluentProvider theme={twillTheme}>
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          height: "100vh",
          overflow: "hidden",
        }}
      >
        <Titlebar />
        <div className="app-container" style={{ flex: 1, minHeight: 0 }}>
          <Sidebar
            activeCategory={activeCategory}
            groups={sidebarGroups}
            onCategoryChange={setActiveCategory}
          />
          <div className="main-workspace">
            <TopHeader
              canSync={runtimeAvailable && accounts.length > 0}
              hasSyncError={syncErrorMessage !== null}
              isSyncing={isSyncing}
              syncSummary={syncSummary}
              onSync={() => {
                void handleSyncWorkspace();
              }}
            />
            <MailWorkspace
              accountsView={
                <AccountsWorkspace
                  accounts={accounts}
                  draft={accountDraft}
                  errorMessage={accountError}
                  isSaving={isSaving}
                  isTesting={isTesting}
                  probeResult={probeResult}
                  runtimeAvailable={runtimeAvailable}
                  onDraftChange={updateDraft}
                  onRefresh={() => {
                    void refreshAccounts();
                  }}
                  onSave={() => {
                    void handleSaveAccount();
                  }}
                  onTest={() => {
                    void handleTestAccount();
                  }}
                />
              }
              category={activeCategory}
              snapshot={workspaceSnapshot}
              onOpenVerificationLink={(url) => openExternalUrl(url)}
            />
          </div>
        </div>
      </div>
    </FluentProvider>
  );
}

export default App;
