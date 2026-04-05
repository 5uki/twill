import { Button, Input, Text } from "@fluentui/react-components";
import { SearchRegular } from "@fluentui/react-icons";

interface TopHeaderProps {
  canSync?: boolean;
  isSyncing?: boolean;
  syncSummary?: string;
  hasSyncError?: boolean;
  onSync?: () => void;
}

export function TopHeader({
  canSync = false,
  isSyncing = false,
  syncSummary = "添加账号后可同步最近邮件",
  hasSyncError = false,
  onSync,
}: TopHeaderProps) {
  return (
    <header className="top-header">
      <Input
        className="top-header-search"
        contentBefore={<SearchRegular />}
        placeholder="搜索邮件、验证码或发件人"
        appearance="outline"
      />
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
