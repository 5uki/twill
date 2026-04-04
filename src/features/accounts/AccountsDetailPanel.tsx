import { Badge, Spinner, Text } from "@fluentui/react-components";
import type { AccountConnectionTestResult, AccountSummary } from "./model";

interface AccountsDetailPanelProps {
  accountCount: number;
  lastSavedAccount: AccountSummary | null;
  lastTestResult: AccountConnectionTestResult | null;
  saveState: "idle" | "submitting";
  testState: "idle" | "submitting";
}

export function AccountsDetailPanel({
  accountCount,
  lastSavedAccount,
  lastTestResult,
  saveState,
  testState,
}: AccountsDetailPanelProps) {
  return (
    <div className="workspace-shell__content-switch">
      <div className="workspace-shell__section-head">
        <div>
          <div className="workspace-shell__section-title">M1 详情</div>
          <Text className="workspace-shell__section-desc">
            当前切片聚焦账户元数据、共享命令链路和连接预检，不在这一刀内引入安全存储。
          </Text>
        </div>
        <Badge appearance="filled" color="success">
          {accountCount} 个账户
        </Badge>
      </div>

      <article className="workspace-shell__detail-card workspace-shell__detail-card--signal">
        <div className="workspace-shell__detail-key">当前切片</div>
        <div className="workspace-shell__detail-value">account add / list / test</div>
        <Text>
          CLI、Tauri command 和桌面 UI 现在共用同一套服务层。下一切片再接密码与系统级安全存储。
        </Text>
      </article>

      {testState === "submitting" ? (
        <article className="workspace-shell__detail-card">
          <Spinner label="正在执行连接预检..." />
        </article>
      ) : null}

      {lastTestResult ? (
        <article className="workspace-shell__detail-card">
          <div className="workspace-shell__detail-metadata">
            <Badge appearance="filled" color={lastTestResult.status === "passed" ? "success" : "warning"}>
              {lastTestResult.status === "passed" ? "预检通过" : "预检失败"}
            </Badge>
            <Badge appearance="outline">规则驱动</Badge>
          </div>
          <Text className="workspace-shell__detail-summary">{lastTestResult.summary}</Text>
          <div className="account-detail__checks">
            {lastTestResult.checks.map((check) => (
              <div key={check.target} className="account-detail__check">
                <div className="workspace-shell__message-topline">
                  <Badge appearance="tint">{check.target.toUpperCase()}</Badge>
                  <Badge appearance="outline">
                    {check.status === "passed" ? "通过" : "失败"}
                  </Badge>
                </div>
                <Text>{check.message}</Text>
              </div>
            ))}
          </div>
        </article>
      ) : null}

      {saveState === "submitting" ? (
        <article className="workspace-shell__detail-card">
          <Spinner label="正在保存账户元数据..." />
        </article>
      ) : null}

      {lastSavedAccount ? (
        <article className="workspace-shell__detail-card">
          <div className="workspace-shell__detail-key">最近保存</div>
          <div className="workspace-shell__detail-value">{lastSavedAccount.display_name}</div>
          <Text>{lastSavedAccount.email}</Text>
          <Text>
            IMAP {lastSavedAccount.imap.host}:{lastSavedAccount.imap.port} / SMTP{" "}
            {lastSavedAccount.smtp.host}:{lastSavedAccount.smtp.port}
          </Text>
        </article>
      ) : null}

      <article className="workspace-shell__detail-card">
        <div className="workspace-shell__detail-key">CLI 模拟器</div>
        <Text className="workspace-shell__detail-summary">
          `account add`, `account list`, `account test` 已作为当前切片的正式验证入口。
        </Text>
      </article>
    </div>
  );
}
