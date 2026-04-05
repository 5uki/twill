import { Avatar, Badge, Text } from "@fluentui/react-components";
import {
  CheckboxUncheckedRegular,
  ChevronLeftRegular,
  ChevronRightRegular,
  DismissRegular,
  MailReadRegular,
  MailUnreadRegular,
  Open16Filled,
  OpenRegular,
  StarRegular,
} from "@fluentui/react-icons";
import {
  useEffect,
  useId,
  useRef,
  useState,
  type FocusEvent,
  type MouseEvent,
  type ReactNode,
} from "react";
import workspaceBootstrap from "../data/workspace-bootstrap.json";
import type {
  WorkspaceMessageAction,
  MessageReadState,
  MessageStatus,
  WorkspaceBootstrapSnapshot,
  WorkspaceExtractItem,
  WorkspaceMessageDetail,
  WorkspaceMessageItem,
  WorkspaceSiteSummary,
} from "../lib/app-types";
import {
  DEFAULT_VERIFICATION_RECENT_HOURS,
  buildWorkspaceMessageFilter,
  filterWorkspaceMessages,
  type WorkspaceMessageCategoryFilter,
} from "../lib/workspace-reading";
import type { WorkspaceCategory } from "../lib/workspace-view";
import { getWorkspaceTitle } from "../lib/workspace-view";
import { getCircularAvatarMetrics } from "./extract-geometry";
import { ExtractTooltipPortal } from "./ExtractTooltipPortal";
import { getExtractActionHint } from "./extract-tooltip";

const fallbackWorkspaceSnapshot =
  workspaceBootstrap as WorkspaceBootstrapSnapshot;

const dateFormatter = new Intl.DateTimeFormat("zh-CN", {
  hour: "2-digit",
  minute: "2-digit",
  month: "2-digit",
  day: "2-digit",
});

const detailDateFormatter = new Intl.DateTimeFormat("zh-CN", {
  month: "2-digit",
  day: "2-digit",
  hour: "2-digit",
  minute: "2-digit",
});

const messageCategoryOptions: {
  value: WorkspaceMessageCategoryFilter;
  label: string;
}[] = [
  { value: "all", label: "全部" },
  { value: "registration", label: "注册" },
  { value: "security", label: "安全" },
  { value: "marketing", label: "营销" },
];

function CircularProgressAvatar({
  name,
  progressPercent,
  label,
}: {
  name: string;
  progressPercent: number;
  label: string;
}) {
  const metrics = getCircularAvatarMetrics();
  const progress = Math.max(0, Math.min(progressPercent, 100)) / 100;
  const circumference = 2 * Math.PI * metrics.radius;
  const strokeDashoffset = circumference - progress * circumference;
  const strokeColor =
    progress < 0.2 ? "#ef4444" : progress < 0.5 ? "#f59e0b" : "#3b82f6";

  return (
    <div className="circular-avatar-container">
      <svg
        className="circular-avatar-svg"
        height={metrics.outerSize}
        width={metrics.outerSize}
      >
        <circle
          cx={metrics.center}
          cy={metrics.center}
          fill="none"
          r={metrics.radius}
          stroke="#e5e7eb"
          strokeWidth={metrics.strokeWidth}
        />
        <circle
          cx={metrics.center}
          cy={metrics.center}
          fill="none"
          r={metrics.radius}
          stroke={strokeColor}
          strokeDasharray={circumference}
          strokeDashoffset={strokeDashoffset}
          strokeLinecap="round"
          strokeWidth={metrics.strokeWidth}
        />
      </svg>
      <div className="circular-avatar-label" style={{ color: strokeColor }}>
        {label}
      </div>
      <div className="circular-avatar-icon">
        <Avatar name={name} size={metrics.innerSize} />
      </div>
    </div>
  );
}

function ExtractCard({
  item,
  onOpenLink,
  onAction,
  onRemove,
}: {
  item: WorkspaceExtractItem;
  onOpenLink: (url: string) => Promise<void> | void;
  onAction?: () => Promise<void> | void;
  onRemove: () => void;
}) {
  const [copied, setCopied] = useState(false);
  const [isClosing, setIsClosing] = useState(false);
  const [isTooltipVisible, setIsTooltipVisible] = useState(false);
  const [tooltipPosition, setTooltipPosition] = useState<{
    left: number;
    top: number;
  } | null>(null);
  const resetCopiedTimer = useRef<number | null>(null);
  const shellRef = useRef<HTMLDivElement>(null);
  const tooltipId = useId();

  useEffect(() => {
    return () => {
      if (resetCopiedTimer.current !== null) {
        window.clearTimeout(resetCopiedTimer.current);
      }
    };
  }, []);

  useEffect(() => {
    if (!isTooltipVisible) {
      return;
    }

    const syncTooltipPosition = () => {
      if (!shellRef.current) {
        return;
      }

      const bounds = shellRef.current.getBoundingClientRect();
      setTooltipPosition({
        left: bounds.left + bounds.width / 2,
        top: bounds.top - 14,
      });
    };

    syncTooltipPosition();
    window.addEventListener("resize", syncTooltipPosition);
    window.addEventListener("scroll", syncTooltipPosition, true);

    return () => {
      window.removeEventListener("resize", syncTooltipPosition);
      window.removeEventListener("scroll", syncTooltipPosition, true);
    };
  }, [isTooltipVisible]);

  const showTooltip = () => {
    if (shellRef.current) {
      const bounds = shellRef.current.getBoundingClientRect();
      setTooltipPosition({
        left: bounds.left + bounds.width / 2,
        top: bounds.top - 14,
      });
    }

    setIsTooltipVisible(true);
  };

  const hideTooltip = () => {
    setIsTooltipVisible(false);
  };

  const handleAction = async () => {
    if (item.kind === "code") {
      await navigator.clipboard.writeText(item.value);
      setCopied(true);

      if (resetCopiedTimer.current !== null) {
        window.clearTimeout(resetCopiedTimer.current);
      }

      resetCopiedTimer.current = window.setTimeout(() => {
        setCopied(false);
        resetCopiedTimer.current = null;
      }, 2000);
      await onAction?.();
      return;
    }

    await onOpenLink(item.value);
    await onAction?.();
  };

  const handleShellBlur = (event: FocusEvent<HTMLDivElement>) => {
    if (
      event.relatedTarget instanceof Node &&
      event.currentTarget.contains(event.relatedTarget)
    ) {
      return;
    }

    hideTooltip();
  };

  const handleClose = (event: MouseEvent<HTMLButtonElement>) => {
    event.stopPropagation();
    setIsClosing(true);
    hideTooltip();
  };

  const handleAnimationEnd = () => {
    if (isClosing) {
      onRemove();
    }
  };

  const handleCloseBlur = (event: FocusEvent<HTMLButtonElement>) => {
    if (
      event.relatedTarget instanceof Node &&
      shellRef.current?.contains(event.relatedTarget)
    ) {
      return;
    }

    hideTooltip();
  };

  const actionText = getExtractActionHint(item.kind, copied);
  const tooltipContent =
    actionText === null ? null : item.kind === "link" ? (
      <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
        <Open16Filled style={{ color: "#60a5fa" }} />
        <span>{actionText}</span>
      </div>
    ) : (
      <span>{actionText}</span>
    );

  return (
    <div
      ref={shellRef}
      className={`extract-minimal-card-shell ${isClosing ? "extract-closing" : ""}`}
      onAnimationEnd={handleAnimationEnd}
      onBlur={handleShellBlur}
      onFocus={showTooltip}
      onMouseEnter={showTooltip}
      onMouseLeave={hideTooltip}
    >
      <button
        aria-describedby={tooltipId}
        className="extract-minimal-card"
        type="button"
        onClick={() => {
          void handleAction();
        }}
      >
        <CircularProgressAvatar
          label={item.expires_label}
          name={item.sender}
          progressPercent={item.progress_percent}
        />
        <div
          className={`extract-minimal-code ${copied ? "copied" : ""} ${
            item.kind === "link" ? "link" : ""
          }`}
        >
          {copied ? "Copied!" : item.kind === "code" ? item.value : item.label}
          {item.kind === "link" && !copied ? (
            <OpenRegular fontSize={14} style={{ marginLeft: 6 }} />
          ) : null}
        </div>
      </button>
      <ExtractTooltipPortal
        id={tooltipId}
        position={tooltipPosition}
        visible={isTooltipVisible && tooltipContent !== null}
      >
        {tooltipContent}
      </ExtractTooltipPortal>
      <button
        aria-label="Dismiss extract item"
        className="extract-minimal-close"
        type="button"
        onBlur={handleCloseBlur}
        onClick={handleClose}
        onFocus={showTooltip}
        onMouseEnter={showTooltip}
      >
        <DismissRegular fontSize={14} />
      </button>
    </div>
  );
}

interface MailWorkspaceProps {
  category: WorkspaceCategory;
  snapshot?: WorkspaceBootstrapSnapshot;
  accountsView?: ReactNode;
  messages?: WorkspaceMessageItem[];
  selectedMessage?: WorkspaceMessageDetail | null;
  selectedMessageId?: string | null;
  isReadingLoading?: boolean;
  messageError?: string | null;
  messageCategoryFilter?: WorkspaceMessageCategoryFilter;
  showOlderVerificationMessages?: boolean;
  onMessageCategoryChange?: (value: WorkspaceMessageCategoryFilter) => void;
  onMessageAction?: (action: WorkspaceMessageAction) => Promise<void> | void;
  onMessageReadStateChange?: (readState: MessageReadState) => Promise<void> | void;
  onMessageStatusChange?: (status: MessageStatus) => void;
  onMessageSelect?: (messageId: string) => void;
  onOpenOriginalMessage?: () => Promise<void> | void;
  onToggleVerificationWindow?: () => void;
  onExtractAction?: (item: WorkspaceExtractItem) => Promise<void> | void;
  onOpenVerificationLink?: (url: string) => Promise<void> | void;
}

export function MailWorkspace({
  category,
  snapshot = fallbackWorkspaceSnapshot,
  accountsView,
  messages,
  selectedMessage,
  selectedMessageId = null,
  isReadingLoading = false,
  messageError = null,
  messageCategoryFilter = "all",
  showOlderVerificationMessages = false,
  onMessageCategoryChange,
  onMessageAction,
  onMessageReadStateChange,
  onMessageStatusChange,
  onMessageSelect,
  onOpenOriginalMessage,
  onToggleVerificationWindow,
  onExtractAction,
  onOpenVerificationLink,
}: MailWorkspaceProps) {
  const [extracts, setExtracts] = useState(snapshot.extracts);
  const scrollRef = useRef<HTMLDivElement>(null);
  const [showLeft, setShowLeft] = useState(false);
  const [showRight, setShowRight] = useState(false);

  useEffect(() => {
    setExtracts(snapshot.extracts);
  }, [snapshot]);

  const checkScroll = () => {
    if (!scrollRef.current) {
      return;
    }

    const { scrollLeft, scrollWidth, clientWidth } = scrollRef.current;
    setShowLeft(scrollLeft > 0);
    setShowRight(Math.ceil(scrollLeft + clientWidth) < scrollWidth);
  };

  useEffect(() => {
    checkScroll();
    window.addEventListener("resize", checkScroll);

    return () => window.removeEventListener("resize", checkScroll);
  }, [extracts]);

  const scroll = (direction: "left" | "right") => {
    if (!scrollRef.current) {
      return;
    }

    scrollRef.current.scrollBy({
      left: direction === "left" ? -300 : 300,
      behavior: "smooth",
    });
  };

  const title = getWorkspaceTitle(category);
  const openLink =
    onOpenVerificationLink ??
    ((url: string) => {
      window.open(url, "_blank", "noopener,noreferrer");
    });

  if (category === "accounts") {
    return (
      <div className="workspace-content">
        <div className="workspace-title-row">
          <Text style={{ fontWeight: 600, fontSize: "18px" }}>{title}</Text>
        </div>
        {accountsView}
      </div>
    );
  }

  if (category === "sites") {
    return (
      <div className="workspace-content">
        <div className="workspace-title-row">
          <Text style={{ fontWeight: 600, fontSize: "18px" }}>{title}</Text>
        </div>
        <div className="site-grid">
          {snapshot.site_summaries.map((site) => (
            <SiteCard key={site.id} site={site} />
          ))}
        </div>
      </div>
    );
  }

  const resolvedMessages = messages ?? getVisibleMessages(snapshot, category);
  const resolvedSelectedMessage =
    selectedMessage ??
    (resolvedMessages.some((message) => message.id === snapshot.selected_message.id)
      ? snapshot.selected_message
      : null);
  const showExtracts = category === "verifications" && extracts.length > 0;
  const emptyTitle =
    category === "verifications" ? "没有匹配的验证邮件" : "没有匹配的收件箱邮件";
  const emptyDescription =
    category === "verifications"
      ? "可以换一个关键字，或切回“全部”分类看看更早的验证消息。"
      : "可以尝试更短的关键字，或切换到其他分类查看同步结果。";

  return (
    <div className="workspace-content">
      {showExtracts ? (
        <div className="extract-zone-wrapper">
          {showLeft ? (
            <button
              className="extract-scroll-button left"
              type="button"
              onClick={() => scroll("left")}
            >
              <ChevronLeftRegular fontSize={24} />
            </button>
          ) : null}
          <div className="extract-zone" ref={scrollRef} onScroll={checkScroll}>
            {extracts.map((item) => (
              <ExtractCard
                key={item.id}
                item={item}
                onAction={() => onExtractAction?.(item)}
                onOpenLink={openLink}
                onRemove={() =>
                  setExtracts((current) =>
                    current.filter((extract) => extract.id !== item.id),
                  )
                }
              />
            ))}
          </div>
          {showRight ? (
            <button
              className="extract-scroll-button right"
              type="button"
              onClick={() => scroll("right")}
            >
              <ChevronRightRegular fontSize={24} />
            </button>
          ) : null}
        </div>
      ) : null}

        <div className="workspace-title-row">
          <div className="workspace-title-stack">
            <Text style={{ fontWeight: 600, fontSize: "18px" }}>{title}</Text>
            <Text className="workspace-subtitle">
              {category === "verifications"
              ? "优先显示验证码、验证链接和需要立刻处理的安全邮件。"
              : "按统一工作台查看最近同步到本地缓存的邮件。"}
            </Text>
          </div>
          <Text className="workspace-count-label">{resolvedMessages.length} 封</Text>
        </div>

      <div className="message-filter-row">
        {messageCategoryOptions.map((option) => {
          const isActive = messageCategoryFilter === option.value;

          return (
            <button
              key={option.value}
              className={`message-filter-chip ${isActive ? "active" : ""}`}
              type="button"
              onClick={() => onMessageCategoryChange?.(option.value)}
            >
              {option.label}
            </button>
          );
        })}
      </div>

      {category === "verifications" ? (
        <div className="workspace-reading-scope">
          <Text className="workspace-reading-scope-label">
            {showOlderVerificationMessages
              ? "已包含更早验证邮件"
              : `最近 ${DEFAULT_VERIFICATION_RECENT_HOURS} 小时`}
          </Text>
          <button
            className="workspace-reading-scope-button"
            disabled={!onToggleVerificationWindow}
            type="button"
            onClick={() => onToggleVerificationWindow?.()}
          >
            {showOlderVerificationMessages
              ? `仅看最近 ${DEFAULT_VERIFICATION_RECENT_HOURS} 小时`
              : "查看更早邮件"}
          </button>
        </div>
      ) : null}

      <div className="reading-layout">
        <section className="reading-list-panel">
          {messageError ? (
            <div className="workspace-inline-alert error">{messageError}</div>
          ) : null}
          {isReadingLoading && resolvedMessages.length > 0 ? (
            <Text className="workspace-loading-label">正在刷新结果…</Text>
          ) : null}
          {resolvedMessages.length === 0 ? (
            <EmptyWorkspaceState
              description={emptyDescription}
              heading={isReadingLoading ? "正在读取邮件…" : emptyTitle}
            />
          ) : (
            <div className="mail-list">
              {resolvedMessages.map((message) => (
                <MessageCard
                  key={message.id}
                  isSelected={message.id === selectedMessageId}
                  message={message}
                  onSelect={() => onMessageSelect?.(message.id)}
                />
              ))}
            </div>
          )}
        </section>

        <aside className="reading-detail-panel">
          {resolvedSelectedMessage ? (
            <MessageDetailCard
              isActionLoading={isReadingLoading}
              message={resolvedSelectedMessage}
              onMessageAction={onMessageAction}
              onMessageReadStateChange={onMessageReadStateChange}
              onOpenOriginalMessage={onOpenOriginalMessage}
              onOpenVerificationLink={openLink}
              onUpdateStatus={onMessageStatusChange}
            />
          ) : (
            <EmptyWorkspaceState
              description="列表结果出来后，点开一封邮件就可以直接查看摘要、正文和验证码动作。"
              heading="选择一封邮件开始阅读"
            />
          )}
        </aside>
      </div>
    </div>
  );
}

function MessageCard({
  message,
  isSelected,
  onSelect,
}: {
  message: WorkspaceMessageItem;
  isSelected: boolean;
  onSelect: () => void;
}) {
  const isUnread = message.read_state === "unread";

  return (
    <button
      className={`mail-item ${isUnread ? "unread" : "read"} ${
        isSelected ? "selected" : ""
      }`}
      type="button"
      onClick={onSelect}
    >
      <div className="mail-item-leading">
        <span className="mail-item-checkbox" aria-hidden="true">
          <CheckboxUncheckedRegular fontSize={18} />
        </span>
        <span
          className={`mail-item-envelope ${isUnread ? "unread" : "read"}`}
          aria-hidden="true"
        >
          {isUnread ? (
            <MailUnreadRegular fontSize={18} />
          ) : (
            <MailReadRegular fontSize={18} />
          )}
        </span>
      </div>

      <div className="mail-item-sender">
        <Text
          className="text-truncate"
          style={{ fontWeight: isUnread ? 600 : 400 }}
        >
          {message.sender}
        </Text>
      </div>

      <div className="mail-item-body">
        <Text
          as="span"
          className={`mail-item-subject ${isUnread ? "unread" : "read"}`}
        >
          {message.subject}
        </Text>
        <Text
          as="span"
          className="text-truncate"
          style={{ color: "#6b7280", fontSize: "13px" }}
        >
          {message.preview}
        </Text>
      </div>

      <div className="mail-item-meta">
        <Text as="span" className="mail-item-date">
          {dateFormatter.format(new Date(message.received_at))}
        </Text>
        <span className="mail-item-star" aria-hidden="true">
          <StarRegular fontSize={18} />
        </span>
      </div>
    </button>
  );
}

function MessageDetailCard({
  isActionLoading = false,
  message,
  onMessageAction,
  onMessageReadStateChange,
  onOpenOriginalMessage,
  onOpenVerificationLink,
  onUpdateStatus,
}: {
  isActionLoading?: boolean;
  message: WorkspaceMessageDetail;
  onMessageAction?: (action: WorkspaceMessageAction) => Promise<void> | void;
  onMessageReadStateChange?: (readState: MessageReadState) => Promise<void> | void;
  onOpenOriginalMessage?: () => Promise<void> | void;
  onOpenVerificationLink: (url: string) => Promise<void> | void;
  onUpdateStatus?: (status: MessageStatus) => void;
}) {
  const [copied, setCopied] = useState(false);

  const handleCopyCode = async () => {
    if (!message.extracted_code) {
      return;
    }

    await navigator.clipboard.writeText(message.extracted_code);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1600);
    await onMessageAction?.("copy_code");
  };

  const handleOpenVerificationLink = async () => {
    if (!message.verification_link) {
      return;
    }

    await onOpenVerificationLink(message.verification_link);
    await onMessageAction?.("open_link");
  };

  return (
    <div className="message-detail-card">
      <div className="message-detail-header">
        <div className="message-detail-heading">
          <Text className="message-detail-subject">{message.subject}</Text>
          <div className="message-detail-badges">
            <Badge appearance="tint">{getMessageStatusLabel(message.status)}</Badge>
            <Badge appearance="outline">{getMessageCategoryLabel(message.category)}</Badge>
          </div>
        </div>
        <Text className="message-detail-meta">
          {message.sender} · {message.account_name} · {message.mailbox_label}
        </Text>
        <Text className="message-detail-meta">
          收到于 {detailDateFormatter.format(new Date(message.received_at))}
        </Text>
      </div>

      <div className="message-detail-actions">
        {message.extracted_code ? (
          <button
            className="message-detail-action"
            disabled={isActionLoading}
            type="button"
            onClick={() => void handleCopyCode()}
          >
            {copied ? "已复制验证码" : `复制验证码 ${message.extracted_code}`}
          </button>
        ) : null}
        {message.verification_link ? (
          <button
            className="message-detail-action secondary"
            disabled={isActionLoading}
            type="button"
            onClick={() => void handleOpenVerificationLink()}
          >
            打开验证链接
          </button>
        ) : null}
        {message.original_message_url ? (
          <button
            className="message-detail-action secondary"
            disabled={isActionLoading}
            type="button"
            onClick={() => void onOpenOriginalMessage?.()}
          >
            打开原始邮件
          </button>
        ) : null}
        <button
          className="message-detail-action secondary"
          disabled={isActionLoading}
          type="button"
          onClick={() =>
            void onMessageReadStateChange?.(
              message.read_state === "unread" ? "read" : "unread",
            )
          }
        >
          {message.read_state === "unread" ? "标记已读" : "标记未读"}
        </button>
        <button
          className="message-detail-action secondary"
          disabled={isActionLoading}
          type="button"
          onClick={() =>
            onUpdateStatus?.(
              message.status === "pending" ? "processed" : "pending",
            )
          }
        >
          {message.status === "pending" ? "标记已处理" : "撤销已处理"}
        </button>
      </div>

      <div className="message-detail-section">
        <Text className="message-detail-section-title">摘要</Text>
        <Text className="message-detail-summary">{message.summary}</Text>
      </div>

      <div className="message-detail-section">
        <Text className="message-detail-section-title">正文</Text>
        <div className="message-detail-body">
          {message.body_text?.trim() || "当前消息只缓存了摘要，还没有完整正文。"}
        </div>
      </div>

      <div className="message-detail-footer">
        <div className="message-detail-footnote">
          <Text className="message-detail-footnote-label">站点提示</Text>
          <Text>{message.site_hint}</Text>
        </div>
        <div className="message-detail-footnote">
          <Text className="message-detail-footnote-label">正文状态</Text>
          <Text>{message.prefetched_body ? "已预抓取" : "仅元数据"}</Text>
        </div>
        <div className="message-detail-footnote">
          <Text className="message-detail-footnote-label">同步时间</Text>
          <Text>{detailDateFormatter.format(new Date(message.synced_at))}</Text>
        </div>
      </div>
    </div>
  );
}

function SiteCard({ site }: { site: WorkspaceSiteSummary }) {
  return (
    <div className="site-card">
      <div className="site-card-header">
        <div>
          <Text className="site-card-title">{site.label}</Text>
          <Text className="site-card-subtitle">{site.hostname}</Text>
        </div>
        <Badge appearance="tint">待处理 {site.pending_count}</Badge>
      </div>
      <Text className="site-card-body">
        最近一封来自 {site.latest_sender}，可以直接回到验证列表继续处理。
      </Text>
    </div>
  );
}

function EmptyWorkspaceState({
  heading,
  description,
}: {
  heading: string;
  description: string;
}) {
  return (
    <div className="workspace-empty-state">
      <Text className="workspace-empty-title">{heading}</Text>
      <Text className="workspace-empty-description">{description}</Text>
    </div>
  );
}

function getMessageStatusLabel(status: WorkspaceMessageDetail["status"]) {
  return status === "pending" ? "待处理" : "已处理";
}

function getMessageCategoryLabel(category: WorkspaceMessageDetail["category"]) {
  if (category === "registration") {
    return "注册";
  }

  if (category === "security") {
    return "安全";
  }

  return "营销";
}

function getVisibleMessages(
  snapshot: WorkspaceBootstrapSnapshot,
  category: WorkspaceCategory,
) {
  if (category === "verifications") {
    return filterWorkspaceMessages(
      snapshot,
      buildWorkspaceMessageFilter("verifications", "all", ""),
    );
  }

  if (category === "inbox") {
    return filterWorkspaceMessages(
      snapshot,
      buildWorkspaceMessageFilter("inbox", "all", ""),
    );
  }

  return [];
}
