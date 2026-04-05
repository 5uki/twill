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
  WorkspaceBootstrapSnapshot,
  WorkspaceExtractItem,
  WorkspaceMessageItem,
  WorkspaceSiteSummary,
} from "../lib/app-types";
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
  onRemove,
}: {
  item: WorkspaceExtractItem;
  onOpenLink: (url: string) => Promise<void> | void;
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
      return;
    }

    await onOpenLink(item.value);
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
  onOpenVerificationLink?: (url: string) => Promise<void> | void;
}

export function MailWorkspace({
  category,
  snapshot = fallbackWorkspaceSnapshot,
  accountsView,
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

  const visibleMessages = getVisibleMessages(snapshot, category);
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

  const showExtracts = category === "verifications" && extracts.length > 0;

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

      <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
        <div className="workspace-title-row">
          <Text style={{ fontWeight: 600, fontSize: "18px" }}>{title}</Text>
        </div>

        <div className="mail-list">
          {visibleMessages.map((mail) => (
            <MessageCard key={mail.id} message={mail} />
          ))}
        </div>
      </div>
    </div>
  );
}

function MessageCard({ message }: { message: WorkspaceMessageItem }) {
  const isUnread = message.status === "pending";

  return (
    <div className={`mail-item ${isUnread ? "unread" : "read"}`}>
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
        <Badge appearance="tint">
          待处理 {site.pending_count}
        </Badge>
      </div>
      <Text className="site-card-body">
        最近一次来自 {site.latest_sender}，可以直接回到验证列表继续处理。
      </Text>
    </div>
  );
}

function getVisibleMessages(
  snapshot: WorkspaceBootstrapSnapshot,
  category: WorkspaceCategory,
) {
  const messages = snapshot.message_groups.flatMap((group) => group.items);

  if (category === "verifications") {
    return messages.filter((message) => message.has_code || message.has_link);
  }

  if (category === "inbox") {
    return messages;
  }

  return [];
}
