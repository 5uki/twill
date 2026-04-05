import { FluentProvider } from "@fluentui/react-components";
import {
  MailInboxRegular,
  PersonAccountsRegular,
  ServerRegular,
  ShieldKeyholeRegular,
} from "@fluentui/react-icons";
import { startTransition, useEffect, useRef, useState } from "react";
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
  applyWorkspaceMessageAction as applyWorkspaceMessageActionFromApi,
  confirmWorkspaceSite as confirmWorkspaceSiteFromApi,
  getErrorMessage,
  hasDesktopRuntime,
  listAccounts,
  listWorkspaceMessages,
  loadWorkspaceBootstrap,
  openExternalUrl,
  openWorkspaceMessage as openWorkspaceMessageFromApi,
  openWorkspaceMessageOriginal as openWorkspaceMessageOriginalFromApi,
  readWorkspaceMessage,
  resolveWorkspaceSiteContext as resolveWorkspaceSiteContextFromApi,
  syncWorkspace,
  testAccountConnection,
  updateWorkspaceMessageReadState,
  updateWorkspaceMessageStatus,
} from "./lib/app-api";
import type {
  AccountConnectionTestResult,
  AccountSummary,
  MessageReadState,
  MessageStatus,
  WorkspaceBootstrapSnapshot,
  WorkspaceExtractItem,
  WorkspaceMessageAction,
  WorkspaceMessageDetail,
  WorkspaceMessageItem,
  WorkspaceSiteContextResolution,
  WorkspaceViewId,
} from "./lib/app-types";
import {
  applyWorkspaceMessageAction as applyWorkspaceMessageActionToSnapshot,
  applyWorkspaceMessageReadState,
  applyWorkspaceMessageStatus,
  buildWorkspaceMessageFilter,
  confirmWorkspaceSite as confirmWorkspaceSiteInSnapshot,
  findWorkspaceMessageIdForExtract,
  filterWorkspaceMessages,
  openWorkspaceMessage as openWorkspaceMessageInSnapshot,
  openWorkspaceMessageOriginal as openWorkspaceMessageOriginalInSnapshot,
  resolveWorkspaceSiteContext as resolveWorkspaceSiteContextFromSnapshot,
  resolveSelectedWorkspaceMessage,
  type WorkspaceMessageCategoryFilter,
} from "./lib/workspace-reading";
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
const fallbackActiveCategory = mapWorkspaceViewToCategory(
  fallbackWorkspaceSnapshot.default_view,
);
const fallbackReadingState = resolveReadingState(
  fallbackWorkspaceSnapshot,
  fallbackActiveCategory,
  "all",
  "",
  null,
  false,
  fallbackWorkspaceSnapshot.selected_message.id,
);

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
    fallbackActiveCategory,
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
  const [currentSiteValue, setCurrentSiteValue] = useState("");
  const [isConfirmingSite, setIsConfirmingSite] = useState(false);
  const [siteResolution, setSiteResolution] =
    useState<WorkspaceSiteContextResolution>(
      resolveWorkspaceSiteContextFromSnapshot(fallbackWorkspaceSnapshot, ""),
    );
  const [searchQuery, setSearchQuery] = useState("");
  const [messageCategoryFilter, setMessageCategoryFilter] =
    useState<WorkspaceMessageCategoryFilter>("all");
  const [showOlderVerificationMessages, setShowOlderVerificationMessages] =
    useState(false);
  const [visibleMessages, setVisibleMessages] = useState<WorkspaceMessageItem[]>(
    fallbackReadingState.messages,
  );
  const [selectedMessage, setSelectedMessage] =
    useState<WorkspaceMessageDetail | null>(fallbackReadingState.selectedMessage);
  const [selectedMessageId, setSelectedMessageId] = useState<string | null>(
    fallbackReadingState.selectedMessageId,
  );
  const [isReadingLoading, setIsReadingLoading] = useState(false);
  const [messageError, setMessageError] = useState<string | null>(null);
  const selectedMessageIdRef = useRef<string | null>(
    fallbackReadingState.selectedMessageId,
  );

  const updateSelectedMessageId = (nextId: string | null) => {
    selectedMessageIdRef.current = nextId;
    setSelectedMessageId(nextId);
  };

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

  useEffect(() => {
    let cancelled = false;

    const localResolution = resolveWorkspaceSiteContextFromSnapshot(
      workspaceSnapshot,
      currentSiteValue,
    );

    startTransition(() => {
      setSiteResolution(localResolution);
    });

    if (!runtimeAvailable) {
      return;
    }

    const loadSiteResolution = async () => {
      try {
        const nextResolution = await resolveWorkspaceSiteContextFromApi(currentSiteValue);

        if (cancelled) {
          return;
        }

        startTransition(() => {
          setSiteResolution(nextResolution);
        });
      } catch {
        if (cancelled) {
          return;
        }
      }
    };

    void loadSiteResolution();

    return () => {
      cancelled = true;
    };
  }, [currentSiteValue, runtimeAvailable, workspaceSnapshot]);

  useEffect(() => {
    let cancelled = false;

    if (!isMailCategory(activeCategory)) {
      startTransition(() => {
        setVisibleMessages([]);
        setSelectedMessage(null);
        updateSelectedMessageId(null);
        setMessageError(null);
      });
      setIsReadingLoading(false);
      return;
    }

    const localState = resolveReadingState(
      workspaceSnapshot,
      activeCategory,
      messageCategoryFilter,
      searchQuery,
      siteResolution.matched_site?.hostname ?? null,
      showOlderVerificationMessages,
      selectedMessageIdRef.current,
    );

    const loadReadingState = async () => {
      setIsReadingLoading(true);

      if (!runtimeAvailable) {
        if (cancelled) {
          return;
        }

        startTransition(() => {
          setVisibleMessages(localState.messages);
          setSelectedMessage(localState.selectedMessage);
          updateSelectedMessageId(localState.selectedMessageId);
          setMessageError(null);
        });
        setIsReadingLoading(false);
        return;
      }

      const filter = buildWorkspaceMessageFilter(
        activeCategory,
        messageCategoryFilter,
        searchQuery,
        siteResolution.matched_site?.hostname ?? null,
        showOlderVerificationMessages ? null : undefined,
      );

      try {
        const messages = await listWorkspaceMessages(filter);
        const nextSelectedId =
          (selectedMessageIdRef.current &&
          messages.some((message) => message.id === selectedMessageIdRef.current)
            ? selectedMessageIdRef.current
            : messages[0]?.id) ?? null;
        const nextSelectedMessage = nextSelectedId
          ? await readWorkspaceMessage(nextSelectedId)
          : null;

        if (cancelled) {
          return;
        }

        startTransition(() => {
          setVisibleMessages(messages);
          setSelectedMessage(nextSelectedMessage);
          updateSelectedMessageId(nextSelectedId);
          setMessageError(null);
        });
      } catch (error) {
        if (cancelled) {
          return;
        }

        startTransition(() => {
          setVisibleMessages(localState.messages);
          setSelectedMessage(localState.selectedMessage);
          updateSelectedMessageId(localState.selectedMessageId);
          setMessageError(getErrorMessage(error));
        });
      } finally {
        if (!cancelled) {
          setIsReadingLoading(false);
        }
      }
    };

    void loadReadingState();

    return () => {
      cancelled = true;
    };
  }, [
    activeCategory,
    messageCategoryFilter,
    runtimeAvailable,
    searchQuery,
    showOlderVerificationMessages,
    siteResolution,
    workspaceSnapshot,
  ]);

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

  const handleSelectMessage = async (messageId: string) => {
    if (!isMailCategory(activeCategory) || messageId === selectedMessageIdRef.current) {
      return;
    }

    updateSelectedMessageId(messageId);

    try {
      setIsReadingLoading(true);
      setMessageError(null);
      const result = runtimeAvailable
        ? await openWorkspaceMessageFromApi(messageId)
        : openWorkspaceMessageInSnapshot(workspaceSnapshot, messageId);
      const localState = resolveReadingState(
        result.snapshot,
        activeCategory,
        messageCategoryFilter,
        searchQuery,
        siteResolution.matched_site?.hostname ?? null,
        showOlderVerificationMessages,
        messageId,
      );

      startTransition(() => {
        setWorkspaceSnapshot(result.snapshot);
        setVisibleMessages(localState.messages);
        setSelectedMessage(localState.selectedMessage ?? result.detail);
        updateSelectedMessageId(localState.selectedMessageId);
      });
    } catch (error) {
      const localState = resolveReadingState(
        workspaceSnapshot,
        activeCategory,
        messageCategoryFilter,
        searchQuery,
        siteResolution.matched_site?.hostname ?? null,
        showOlderVerificationMessages,
        messageId,
      );

      startTransition(() => {
        setSelectedMessage(localState.selectedMessage);
        updateSelectedMessageId(localState.selectedMessageId);
        setMessageError(getErrorMessage(error));
      });
    } finally {
      setIsReadingLoading(false);
    }
  };

  const handleUpdateMessageStatus = async (status: MessageStatus) => {
    if (!isMailCategory(activeCategory) || !selectedMessageIdRef.current) {
      return;
    }

    const messageId = selectedMessageIdRef.current;

    try {
      setIsReadingLoading(true);
      setMessageError(null);
      const nextSnapshot = runtimeAvailable
        ? await updateWorkspaceMessageStatus(messageId, status)
        : applyWorkspaceMessageStatus(workspaceSnapshot, messageId, status);
      const localState = resolveReadingState(
        nextSnapshot,
        activeCategory,
        messageCategoryFilter,
        searchQuery,
        siteResolution.matched_site?.hostname ?? null,
        showOlderVerificationMessages,
        messageId,
      );

      startTransition(() => {
        setWorkspaceSnapshot(nextSnapshot);
        setVisibleMessages(localState.messages);
        setSelectedMessage(localState.selectedMessage);
        updateSelectedMessageId(localState.selectedMessageId);
      });
    } catch (error) {
      setMessageError(getErrorMessage(error));
    } finally {
      setIsReadingLoading(false);
    }
  };

  const handleUpdateMessageReadState = async (readState: MessageReadState) => {
    if (!isMailCategory(activeCategory) || !selectedMessageIdRef.current) {
      return;
    }

    const messageId = selectedMessageIdRef.current;

    try {
      setIsReadingLoading(true);
      setMessageError(null);
      const nextSnapshot = runtimeAvailable
        ? await updateWorkspaceMessageReadState(messageId, readState)
        : applyWorkspaceMessageReadState(workspaceSnapshot, messageId, readState);
      const localState = resolveReadingState(
        nextSnapshot,
        activeCategory,
        messageCategoryFilter,
        searchQuery,
        siteResolution.matched_site?.hostname ?? null,
        showOlderVerificationMessages,
        messageId,
      );

      startTransition(() => {
        setWorkspaceSnapshot(nextSnapshot);
        setVisibleMessages(localState.messages);
        setSelectedMessage(localState.selectedMessage);
        updateSelectedMessageId(localState.selectedMessageId);
      });
    } catch (error) {
      setMessageError(getErrorMessage(error));
    } finally {
      setIsReadingLoading(false);
    }
  };

  const handleWorkspaceMessageAction = async (
    messageId: string,
    action: WorkspaceMessageAction,
  ) => {
    if (!isMailCategory(activeCategory)) {
      return;
    }

    try {
      setIsReadingLoading(true);
      setMessageError(null);
      const result = runtimeAvailable
        ? await applyWorkspaceMessageActionFromApi(messageId, action)
        : applyWorkspaceMessageActionToSnapshot(
            workspaceSnapshot,
            messageId,
            action,
          );
      const localState = resolveReadingState(
        result.snapshot,
        activeCategory,
        messageCategoryFilter,
        searchQuery,
        siteResolution.matched_site?.hostname ?? null,
        showOlderVerificationMessages,
        messageId,
      );

      startTransition(() => {
        setWorkspaceSnapshot(result.snapshot);
        setVisibleMessages(localState.messages);
        setSelectedMessage(localState.selectedMessage);
        updateSelectedMessageId(localState.selectedMessageId);
      });
    } catch (error) {
      setMessageError(getErrorMessage(error));
    } finally {
      setIsReadingLoading(false);
    }
  };

  const handleOpenOriginalMessage = async () => {
    if (!isMailCategory(activeCategory) || !selectedMessageIdRef.current) {
      return;
    }

    const messageId = selectedMessageIdRef.current;

    try {
      setIsReadingLoading(true);
      setMessageError(null);
      const result = runtimeAvailable
        ? await openWorkspaceMessageOriginalFromApi(messageId)
        : openWorkspaceMessageOriginalInSnapshot(workspaceSnapshot, messageId);
      const localState = resolveReadingState(
        result.snapshot,
        activeCategory,
        messageCategoryFilter,
        searchQuery,
        siteResolution.matched_site?.hostname ?? null,
        showOlderVerificationMessages,
        messageId,
      );

      startTransition(() => {
        setWorkspaceSnapshot(result.snapshot);
        setVisibleMessages(localState.messages);
        setSelectedMessage(localState.selectedMessage);
        updateSelectedMessageId(localState.selectedMessageId);
      });

      await openExternalUrl(result.original_url);
    } catch (error) {
      setMessageError(getErrorMessage(error));
    } finally {
      setIsReadingLoading(false);
    }
  };

  const handleExtractAction = async (item: WorkspaceExtractItem) => {
    const messageId = findWorkspaceMessageIdForExtract(workspaceSnapshot, item);

    if (!messageId) {
      setMessageError("未找到提取项对应的邮件，无法自动更新状态。");
      return;
    }

    await handleWorkspaceMessageAction(
      messageId,
      item.kind === "code" ? "copy_code" : "open_link",
    );
  };

  const handleConfirmWorkspaceSite = async () => {
    const confirmInput = siteResolution.normalized_domain ?? currentSiteValue;

    if (!confirmInput.trim()) {
      return;
    }

    try {
      setIsConfirmingSite(true);
      setMessageError(null);
      const nextSnapshot = runtimeAvailable
        ? await confirmWorkspaceSiteFromApi(confirmInput)
        : confirmWorkspaceSiteInSnapshot(workspaceSnapshot, confirmInput);
      const nextSiteValue =
        resolveWorkspaceSiteContextFromSnapshot(nextSnapshot, confirmInput)
          .normalized_domain ?? confirmInput;

      startTransition(() => {
        setWorkspaceSnapshot(nextSnapshot);
        setCurrentSiteValue(nextSiteValue);
      });
    } catch (error) {
      setMessageError(getErrorMessage(error));
    } finally {
      setIsConfirmingSite(false);
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
              currentSiteValue={currentSiteValue}
              hasSyncError={syncErrorMessage !== null}
              isSyncing={isSyncing}
              isConfirmingSite={isConfirmingSite}
              searchValue={searchQuery}
              siteResolution={siteResolution}
              onCurrentSiteChange={setCurrentSiteValue}
              onConfirmSite={() => {
                void handleConfirmWorkspaceSite();
              }}
              onCurrentSiteSelect={setCurrentSiteValue}
              syncSummary={syncSummary}
              onSearchChange={setSearchQuery}
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
              messageCategoryFilter={messageCategoryFilter}
              messageError={messageError}
              messages={visibleMessages}
              selectedMessage={selectedMessage}
              selectedMessageId={selectedMessageId}
              showOlderVerificationMessages={showOlderVerificationMessages}
              snapshot={workspaceSnapshot}
              isReadingLoading={isReadingLoading}
              onMessageCategoryChange={setMessageCategoryFilter}
              onMessageAction={async (action) => {
                if (!selectedMessageIdRef.current) {
                  return;
                }

                await handleWorkspaceMessageAction(
                  selectedMessageIdRef.current,
                  action,
                );
              }}
              onExtractAction={handleExtractAction}
              onMessageStatusChange={(status) => {
                void handleUpdateMessageStatus(status);
              }}
              onMessageReadStateChange={(readState) => {
                void handleUpdateMessageReadState(readState);
              }}
              onMessageSelect={(messageId) => {
                void handleSelectMessage(messageId);
              }}
              onOpenOriginalMessage={() => {
                void handleOpenOriginalMessage();
              }}
              onToggleVerificationWindow={() => {
                setShowOlderVerificationMessages((current) => !current);
              }}
              onOpenVerificationLink={(url) => openExternalUrl(url)}
            />
          </div>
        </div>
      </div>
    </FluentProvider>
  );
}

function isMailCategory(
  category: WorkspaceCategory,
): category is "verifications" | "inbox" {
  return category === "verifications" || category === "inbox";
}

function resolveReadingState(
  snapshot: WorkspaceBootstrapSnapshot,
  category: WorkspaceCategory,
  messageCategoryFilter: WorkspaceMessageCategoryFilter,
  searchQuery: string,
  siteHint: string | null,
  showOlderVerificationMessages: boolean,
  preferredMessageId: string | null,
) {
  if (!isMailCategory(category)) {
    return {
      messages: [] as WorkspaceMessageItem[],
      selectedMessage: null as WorkspaceMessageDetail | null,
      selectedMessageId: null as string | null,
    };
  }

  const filter = buildWorkspaceMessageFilter(
    category,
    messageCategoryFilter,
    searchQuery,
    siteHint,
    showOlderVerificationMessages ? null : undefined,
  );
  const messages = filterWorkspaceMessages(snapshot, filter);
  const selectedMessage = resolveSelectedWorkspaceMessage(
    snapshot,
    messages,
    preferredMessageId,
  );

  return {
    messages,
    selectedMessage,
    selectedMessageId: selectedMessage?.id ?? null,
  };
}

export default App;
