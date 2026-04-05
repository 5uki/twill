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

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      const [nextWorkspaceSnapshot, nextAccounts] = await Promise.all([
        loadWorkspaceBootstrap(),
        listAccounts(),
      ]);

      if (cancelled) {
        return;
      }

      startTransition(() => {
        setWorkspaceSnapshot(nextWorkspaceSnapshot);
        setActiveCategory(
          mapWorkspaceViewToCategory(nextWorkspaceSnapshot.default_view),
        );
        setAccounts(nextAccounts);
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

      startTransition(() => {
        setAccounts(nextAccounts);
        setAccountDraft(createEmptyAccountFormDraft());
        setProbeResult(null);
      });
    } catch (error) {
      setAccountError(getErrorMessage(error));
    } finally {
      setIsSaving(false);
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
            <TopHeader />
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
