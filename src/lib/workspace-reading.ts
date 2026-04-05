import type {
  MessageCategory,
  MessageReadState,
  MessageStatus,
  WorkspaceBootstrapSnapshot,
  WorkspaceMailboxSummary,
  WorkspaceMessageAction,
  WorkspaceMessageActionResult,
  WorkspaceMessageDetail,
  WorkspaceMessageFilter,
  WorkspaceMessageGroup,
  WorkspaceMailboxKind,
  WorkspaceExtractItem,
  WorkspaceMessageItem,
  WorkspaceMessageOpenResult,
  WorkspaceMessageOriginalOpenResult,
  WorkspaceSiteContextResolution,
  WorkspaceSiteSummary,
} from "./app-types";

export const DEFAULT_VERIFICATION_RECENT_HOURS = 48;

export function filterWorkspaceMessages(
  snapshot: WorkspaceBootstrapSnapshot,
  filter: WorkspaceMessageFilter = {},
): WorkspaceMessageItem[] {
  const siteHintByMessageId = buildMessageSiteHintMap(
    snapshot.message_details,
    snapshot.selected_message,
  );
  const normalizedSiteHint = normalizeSiteInput(filter.site_hint);

  return snapshot.message_groups.flatMap((group) => group.items).filter((item) => {
    if (filter.account_id && item.account_id !== filter.account_id) {
      return false;
    }

    if (
      filter.mailbox_kind &&
      !matchesMailboxKind(item.mailbox_label, filter.mailbox_kind)
    ) {
      return false;
    }

    if (filter.verification_only && !(item.has_code || item.has_link)) {
      return false;
    }

    if (filter.category && item.category !== filter.category) {
      return false;
    }

    if (normalizedSiteHint && siteHintByMessageId.get(item.id) !== normalizedSiteHint) {
      return false;
    }

    if (!matchesRecentWindow(item.received_at, snapshot.generated_at, filter.recent_hours)) {
      return false;
    }

    return matchesQuery(item, filter.query);
  });
}

export function readWorkspaceMessageFromSnapshot(
  snapshot: WorkspaceBootstrapSnapshot,
  messageId: string,
): WorkspaceMessageDetail {
  const detail = snapshot.message_details.find((item) => item.id === messageId);

  if (detail) {
    return detail;
  }

  if (snapshot.selected_message.id === messageId) {
    return snapshot.selected_message;
  }

  throw new Error(`未找到消息 ${messageId}`);
}

export function resolveSelectedWorkspaceMessage(
  snapshot: WorkspaceBootstrapSnapshot,
  messages: WorkspaceMessageItem[],
  preferredMessageId?: string | null,
): WorkspaceMessageDetail | null {
  if (messages.length === 0) {
    return null;
  }

  const visibleIds = new Set(messages.map((message) => message.id));
  const targetId =
    (preferredMessageId && visibleIds.has(preferredMessageId)
      ? preferredMessageId
      : null) ??
    (visibleIds.has(snapshot.selected_message.id) ? snapshot.selected_message.id : null) ??
    messages[0]?.id;

  if (!targetId) {
    return null;
  }

  return readWorkspaceMessageFromSnapshot(snapshot, targetId);
}

export function applyWorkspaceMessageStatus(
  snapshot: WorkspaceBootstrapSnapshot,
  messageId: string,
  status: MessageStatus,
): WorkspaceBootstrapSnapshot {
  const nextSnapshot = structuredClone(snapshot);
  const items = nextSnapshot.message_groups.flatMap((group) => group.items);
  let found = false;

  for (const item of items) {
    if (item.id === messageId) {
      item.status = status;
      if (status === "processed") {
        item.read_state = "read";
      }
      found = true;
    }
  }

  for (const detail of nextSnapshot.message_details) {
    if (detail.id === messageId) {
      detail.status = status;
      if (status === "processed") {
        detail.read_state = "read";
      }
      found = true;
    }
  }

  if (nextSnapshot.selected_message.id === messageId) {
    nextSnapshot.selected_message.status = status;
    if (status === "processed") {
      nextSnapshot.selected_message.read_state = "read";
    }
    found = true;
  }

  if (!found) {
    throw new Error(`未找到消息 ${messageId}`);
  }

  nextSnapshot.message_groups = rebuildMessageGroups(items, nextSnapshot.message_groups);
  nextSnapshot.mailboxes = rebuildMailboxes(nextSnapshot.message_groups);
  nextSnapshot.site_summaries = rebuildSiteSummaries(
    nextSnapshot.site_summaries,
    nextSnapshot.message_details,
    nextSnapshot.selected_message,
  );

  return nextSnapshot;
}

export function applyWorkspaceMessageReadState(
  snapshot: WorkspaceBootstrapSnapshot,
  messageId: string,
  readState: MessageReadState,
): WorkspaceBootstrapSnapshot {
  const nextSnapshot = structuredClone(snapshot);
  const items = nextSnapshot.message_groups.flatMap((group) => group.items);
  let found = false;

  for (const item of items) {
    if (item.id === messageId) {
      item.read_state = readState;
      found = true;
    }
  }

  for (const detail of nextSnapshot.message_details) {
    if (detail.id === messageId) {
      detail.read_state = readState;
      found = true;
    }
  }

  if (nextSnapshot.selected_message.id === messageId) {
    nextSnapshot.selected_message.read_state = readState;
    found = true;
  }

  if (!found) {
    throw new Error(`未找到消息 ${messageId}`);
  }

  nextSnapshot.message_groups = rebuildMessageGroups(items, nextSnapshot.message_groups);
  nextSnapshot.mailboxes = rebuildMailboxes(nextSnapshot.message_groups);
  nextSnapshot.site_summaries = rebuildSiteSummaries(
    nextSnapshot.site_summaries,
    nextSnapshot.message_details,
    nextSnapshot.selected_message,
  );

  return nextSnapshot;
}

export function applyWorkspaceMessageAction(
  snapshot: WorkspaceBootstrapSnapshot,
  messageId: string,
  action: WorkspaceMessageAction,
): WorkspaceMessageActionResult {
  const detail = readWorkspaceMessageFromSnapshot(snapshot, messageId);
  const nextSnapshot = applyWorkspaceMessageStatus(snapshot, messageId, "processed");

  if (action === "copy_code") {
    if (!detail.extracted_code) {
      throw new Error(`消息 ${messageId} 没有可复制的验证码`);
    }

    nextSnapshot.extracts = nextSnapshot.extracts.filter(
      (extract) => !(extract.kind === "code" && extract.value === detail.extracted_code),
    );

    return {
      action,
      message_id: messageId,
      copied_value: detail.extracted_code,
      opened_url: null,
      snapshot: nextSnapshot,
    };
  }

  if (!detail.verification_link) {
    throw new Error(`消息 ${messageId} 没有可打开的验证链接`);
  }

  nextSnapshot.extracts = nextSnapshot.extracts.filter(
    (extract) => !(extract.kind === "link" && extract.value === detail.verification_link),
  );

  return {
    action,
    message_id: messageId,
    copied_value: null,
    opened_url: detail.verification_link,
    snapshot: nextSnapshot,
  };
}

export function openWorkspaceMessage(
  snapshot: WorkspaceBootstrapSnapshot,
  messageId: string,
): WorkspaceMessageOpenResult {
  const nextSnapshot = applyWorkspaceMessageReadState(snapshot, messageId, "read");

  return {
    detail: readWorkspaceMessageFromSnapshot(nextSnapshot, messageId),
    snapshot: nextSnapshot,
  };
}

export function openWorkspaceMessageOriginal(
  snapshot: WorkspaceBootstrapSnapshot,
  messageId: string,
): WorkspaceMessageOriginalOpenResult {
  const detail = readWorkspaceMessageFromSnapshot(snapshot, messageId);

  if (!detail.original_message_url) {
    throw new Error(`消息 ${messageId} 没有可打开的原始邮件入口`);
  }

  return {
    message_id: messageId,
    original_url: detail.original_message_url,
    snapshot: applyWorkspaceMessageReadState(snapshot, messageId, "read"),
  };
}

export function confirmWorkspaceSite(
  snapshot: WorkspaceBootstrapSnapshot,
  input: string,
  label?: string | null,
): WorkspaceBootstrapSnapshot {
  const normalizedDomain = normalizeSiteInput(input);

  if (!normalizedDomain) {
    throw new Error("请输入可识别的站点域名");
  }

  const nextSnapshot = structuredClone(snapshot);
  if (!isConfirmableSiteDomain(normalizedDomain)) {
    throw new Error(`请输入完整的站点域名，当前值为 ${normalizedDomain}`);
  }

  const siteLabel = label?.trim() || normalizedDomain;
  const existing = nextSnapshot.site_summaries.find(
    (site) => site.hostname.toLowerCase() === normalizedDomain,
  );

  if (existing) {
    existing.label = siteLabel;
    return nextSnapshot;
  }

  nextSnapshot.site_summaries.push({
    id: `site_${normalizedDomain.replace(/[.@]/g, "_")}`,
    label: siteLabel,
    hostname: normalizedDomain,
    pending_count: 0,
    latest_sender: "",
  });

  return nextSnapshot;
}

export function findWorkspaceMessageIdForExtract(
  snapshot: WorkspaceBootstrapSnapshot,
  extract: WorkspaceExtractItem,
) {
  const detail = dedupeDetails(snapshot.message_details, snapshot.selected_message).find(
    (item) =>
      (extract.kind === "code" && item.extracted_code === extract.value) ||
      (extract.kind === "link" && item.verification_link === extract.value),
  );

  return detail?.id ?? null;
}

export function resolveWorkspaceSiteContext(
  snapshot: WorkspaceBootstrapSnapshot,
  input: string,
): WorkspaceSiteContextResolution {
  const normalizedDomain = normalizeSiteInput(input);

  if (!normalizedDomain) {
    return {
      input,
      normalized_domain: null,
      matched_site: null,
      candidate_sites: [],
    };
  }

  const matchedSite =
    snapshot.site_summaries.find(
      (site) => site.hostname.toLowerCase() === normalizedDomain,
    ) ?? null;
  const candidateSites = matchedSite
    ? []
    : snapshot.site_summaries
        .filter((site) => matchesSiteCandidate(site, normalizedDomain))
        .sort((left, right) => {
          const scoreDiff =
            getSiteCandidateScore(right, normalizedDomain) -
            getSiteCandidateScore(left, normalizedDomain);

          if (scoreDiff !== 0) {
            return scoreDiff;
          }

          if (right.pending_count !== left.pending_count) {
            return right.pending_count - left.pending_count;
          }

          return left.hostname.localeCompare(right.hostname);
        });

  return {
    input,
    normalized_domain: normalizedDomain,
    matched_site: matchedSite,
    candidate_sites: candidateSites,
  };
}

function matchesMailboxKind(
  mailboxLabel: string,
  mailboxKind: WorkspaceMailboxKind,
): boolean {
  if (mailboxKind === "inbox") {
    return mailboxLabel === "Inbox";
  }

  return mailboxLabel === "Spam/Junk";
}

function matchesQuery(
  item: WorkspaceMessageItem,
  query: string | null | undefined,
): boolean {
  const normalizedQuery = query?.trim().toLowerCase();

  if (!normalizedQuery) {
    return true;
  }

  return [
    item.subject,
    item.sender,
    item.preview,
    item.account_name,
    item.mailbox_label,
  ].some((value) => value.toLowerCase().includes(normalizedQuery));
}

function matchesRecentWindow(
  receivedAt: string,
  generatedAt: string,
  recentHours: number | null | undefined,
) {
  if (!recentHours) {
    return true;
  }

  const receivedTimestamp = parseSnapshotTimestamp(receivedAt);
  const generatedTimestamp = parseSnapshotTimestamp(generatedAt);

  if (receivedTimestamp === null || generatedTimestamp === null) {
    return false;
  }

  return receivedTimestamp >= generatedTimestamp - recentHours * 60 * 60 * 1000;
}

function parseSnapshotTimestamp(value: string) {
  const trimmed = value.trim();

  if (!trimmed) {
    return null;
  }

  const numericTimestamp = Number(trimmed);

  if (Number.isFinite(numericTimestamp)) {
    return numericTimestamp * 1000;
  }

  const parsed = Date.parse(trimmed);

  return Number.isFinite(parsed) ? parsed : null;
}

function buildMessageSiteHintMap(
  messageDetails: WorkspaceMessageDetail[],
  selectedMessage: WorkspaceMessageDetail,
) {
  return dedupeDetails(messageDetails, selectedMessage).reduce((map, detail) => {
    const siteHint = normalizeSiteInput(detail.site_hint);

    if (siteHint) {
      map.set(detail.id, siteHint);
    }

    return map;
  }, new Map<string, string>());
}

function normalizeSiteInput(value: string | null | undefined) {
  const trimmed = value?.trim().toLowerCase();

  if (!trimmed) {
    return null;
  }

  const withoutScheme = trimmed.includes("://")
    ? trimmed.split("://")[1] ?? ""
    : trimmed;
  const authority = withoutScheme.split(/[/?#]/)[0] ?? "";
  const authorityParts = authority.split("@");
  const withoutUserInfo =
    authority.includes("@") && authorityParts.length > 0
      ? authorityParts[authorityParts.length - 1] ?? authority
      : authority;
  const withoutWww = withoutUserInfo.startsWith("www.")
    ? withoutUserInfo.slice(4)
    : withoutUserInfo;
  const hostname = (withoutWww.split(":")[0] ?? "").replace(/^\.+|\.+$/g, "");

  return hostname.length > 0 ? hostname : null;
}

function isConfirmableSiteDomain(hostname: string) {
  return hostname.includes(".");
}

function matchesSiteCandidate(site: WorkspaceSiteSummary, query: string) {
  const hostname = site.hostname.toLowerCase();
  const label = site.label.toLowerCase();

  return hostname.includes(query) || label.includes(query) || query.includes(hostname);
}

function getSiteCandidateScore(site: WorkspaceSiteSummary, query: string) {
  const hostname = site.hostname.toLowerCase();
  const label = site.label.toLowerCase();

  if (hostname.startsWith(query)) {
    return 3;
  }

  if (label.startsWith(query)) {
    return 2;
  }

  return 1;
}

function rebuildMessageGroups(
  items: WorkspaceMessageItem[],
  existingGroups: WorkspaceMessageGroup[],
): WorkspaceMessageGroup[] {
  const groupLabels = new Map(existingGroups.map((group) => [group.id, group.label]));
  const pendingItems: WorkspaceMessageItem[] = [];
  const processedItems: WorkspaceMessageItem[] = [];

  for (const item of items) {
    if (item.status === "pending") {
      pendingItems.push(item);
      continue;
    }

    processedItems.push(item);
  }

  return [
    {
      id: "pending",
      label: groupLabels.get("pending") ?? "待处理",
      items: pendingItems,
    },
    {
      id: "processed",
      label: groupLabels.get("processed") ?? "已处理",
      items: processedItems,
    },
  ].filter((group) => group.items.length > 0);
}

function rebuildMailboxes(
  messageGroups: WorkspaceMessageGroup[],
): WorkspaceMailboxSummary[] {
  const mailboxes = new Map<string, WorkspaceMailboxSummary>();

  for (const item of messageGroups.flatMap((group) => group.items)) {
    const mailboxKind = matchesMailboxKind(item.mailbox_label, "spam_junk")
      ? "spam_junk"
      : "inbox";
    const mailboxKey = `${item.account_id}:${mailboxKind}`;
    const mailbox = mailboxes.get(mailboxKey) ?? {
      id: item.mailbox_id,
      account_id: item.account_id,
      account_name: item.account_name,
      label: item.mailbox_label,
      kind: mailboxKind,
      total_count: 0,
      unread_count: 0,
      verification_count: 0,
    };

    mailbox.total_count += 1;
    mailbox.unread_count += item.read_state === "unread" ? 1 : 0;
    mailbox.verification_count += item.has_code || item.has_link ? 1 : 0;
    mailboxes.set(mailboxKey, mailbox);
  }

  return Array.from(mailboxes.values());
}

function rebuildSiteSummaries(
  existingSummaries: WorkspaceSiteSummary[],
  messageDetails: WorkspaceMessageDetail[],
  selectedMessage: WorkspaceMessageDetail,
): WorkspaceSiteSummary[] {
  const metrics = new Map<
    string,
    { latestReceivedAt: string; latestSender: string; pendingCount: number }
  >();

  for (const detail of dedupeDetails(messageDetails, selectedMessage)) {
    const current = metrics.get(detail.site_hint) ?? {
      latestReceivedAt: "",
      latestSender: detail.sender,
      pendingCount: 0,
    };

    if (detail.status === "pending") {
      current.pendingCount += 1;
    }

    if (detail.received_at >= current.latestReceivedAt) {
      current.latestReceivedAt = detail.received_at;
      current.latestSender = detail.sender;
    }

    metrics.set(detail.site_hint, current);
  }

  const summaries = existingSummaries
    .map((summary) => {
      const siteMetrics = metrics.get(summary.hostname);

      if (!siteMetrics) {
        return {
          ...summary,
          pending_count: 0,
        };
      }

      metrics.delete(summary.hostname);

      return {
        ...summary,
        pending_count: siteMetrics.pendingCount,
        latest_sender: siteMetrics.latestSender,
      };
    });

  for (const [hostname, siteMetrics] of metrics) {
    summaries.push({
      id: `site_${hostname.replace(/[.@]/g, "_")}`,
      label: hostname,
      hostname,
      pending_count: siteMetrics.pendingCount,
      latest_sender: siteMetrics.latestSender,
    });
  }

  return summaries;
}

function dedupeDetails(
  messageDetails: WorkspaceMessageDetail[],
  selectedMessage: WorkspaceMessageDetail,
) {
  const details = [...messageDetails];

  if (!details.some((detail) => detail.id === selectedMessage.id)) {
    details.push(selectedMessage);
  }

  return details;
}

export type WorkspaceMessageCategoryFilter = MessageCategory | "all";

export function buildWorkspaceMessageFilter(
  category: "verifications" | "inbox",
  messageCategory: WorkspaceMessageCategoryFilter,
  query: string,
  siteHint: string | null = null,
  recentHours?: number | null,
): WorkspaceMessageFilter {
  const resolvedRecentHours =
    recentHours === undefined
      ? category === "verifications"
        ? DEFAULT_VERIFICATION_RECENT_HOURS
        : null
      : recentHours;

  return {
    verification_only: category === "verifications",
    category: messageCategory === "all" ? null : messageCategory,
    site_hint: siteHint,
    query,
    recent_hours: resolvedRecentHours,
  };
}
