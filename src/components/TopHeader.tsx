import { Button, Input, Text } from "@fluentui/react-components";
import { SearchRegular } from "@fluentui/react-icons";
import type { WorkspaceSiteContextResolution } from "../lib/app-types";

interface TopHeaderProps {
  canSync?: boolean;
  isSyncing?: boolean;
  isConfirmingSite?: boolean;
  syncSummary?: string;
  hasSyncError?: boolean;
  currentSiteValue?: string;
  searchValue?: string;
  siteResolution?: WorkspaceSiteContextResolution | null;
  onCurrentSiteChange?: (value: string) => void;
  onConfirmSite?: () => void;
  onCurrentSiteSelect?: (hostname: string) => void;
  onSearchChange?: (value: string) => void;
  onSync?: () => void;
}

function isConfirmableSiteDomain(hostname: string) {
  return hostname.includes(".");
}

export function TopHeader({
  canSync = false,
  isSyncing = false,
  isConfirmingSite = false,
  syncSummary = "添加账号后即可同步最近邮件",
  hasSyncError = false,
  currentSiteValue = "",
  searchValue = "",
  siteResolution = null,
  onCurrentSiteChange,
  onConfirmSite,
  onCurrentSiteSelect,
  onSearchChange,
  onSync,
}: TopHeaderProps) {
  const helperText = getSiteHelperText(currentSiteValue, siteResolution);
  const candidateSites = siteResolution?.matched_site
    ? []
    : siteResolution?.candidate_sites ?? [];
  const shouldShowConfirmAction = Boolean(
    siteResolution?.normalized_domain &&
      !siteResolution.matched_site &&
      isConfirmableSiteDomain(siteResolution.normalized_domain),
  );

  return (
    <header className="top-header">
      <div className="top-header-inputs">
        <div className="top-header-site-block">
          <Input
            className="top-header-site"
            appearance="outline"
            placeholder="输入当前站点，如 github.com"
            value={currentSiteValue}
            onChange={(_, data) => onCurrentSiteChange?.(data.value)}
          />
          <div className="top-header-site-meta">
            <Text
              className={`top-header-site-status ${
                siteResolution?.matched_site ? "is-match" : ""
              }`}
            >
              {helperText}
            </Text>
            {shouldShowConfirmAction ? (
              <button
                className="top-header-site-chip confirm"
                disabled={isConfirmingSite || !onConfirmSite}
                type="button"
                onClick={onConfirmSite}
              >
                {isConfirmingSite ? "加入中…" : "加入网站清单"}
              </button>
            ) : null}
            {candidateSites.length > 0 ? (
              <div className="top-header-site-candidates">
                {candidateSites.map((site) => (
                  <button
                    key={site.id}
                    className="top-header-site-chip"
                    type="button"
                    onClick={() => onCurrentSiteSelect?.(site.hostname)}
                  >
                    {site.hostname}
                  </button>
                ))}
              </div>
            ) : null}
          </div>
        </div>
        <Input
          className="top-header-search"
          contentBefore={<SearchRegular />}
          placeholder="搜索邮件、验证码或发件人"
          appearance="outline"
          value={searchValue}
          onChange={(_, data) => onSearchChange?.(data.value)}
        />
      </div>
      <div className="top-header-actions">
        <Text
          className={`top-header-sync-summary ${hasSyncError ? "is-error" : ""}`}
        >
          {syncSummary}
        </Text>
        <Button
          appearance="secondary"
          disabled={!canSync || isSyncing}
          onClick={onSync}
        >
          {isSyncing ? "同步中" : "立即同步"}
        </Button>
      </div>
    </header>
  );
}

function getSiteHelperText(
  currentSiteValue: string,
  siteResolution: WorkspaceSiteContextResolution | null,
) {
  if (!currentSiteValue.trim()) {
    return "输入当前站点后，会优先筛选相关消息";
  }

  if (siteResolution?.matched_site) {
    return `已匹配 ${siteResolution.matched_site.label} (${siteResolution.matched_site.hostname})`;
  }

  if ((siteResolution?.candidate_sites.length ?? 0) > 0) {
    return `未精确命中 ${siteResolution?.normalized_domain ?? currentSiteValue}，可试试候选站点`;
  }

  return `暂未找到 ${siteResolution?.normalized_domain ?? currentSiteValue} 的站点匹配`;
}
