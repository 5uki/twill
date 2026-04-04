import {
  Badge,
  Button,
  MessageBar,
  MessageBarBody,
  Spinner,
  Text,
  Title3,
} from "@fluentui/react-components";
import type { AccountFormState } from "./form";
import type { AccountConnectionTestResult, AccountSummary } from "./model";

interface AccountsWorkspacePanelProps {
  form: AccountFormState;
  accountsState:
    | { status: "loading"; accounts: AccountSummary[] }
    | { status: "ready"; accounts: AccountSummary[] }
    | { status: "error"; accounts: AccountSummary[]; message: string };
  saveState: "idle" | "submitting";
  testState: "idle" | "submitting";
  saveError: string | null;
  testError: string | null;
  lastSavedAccount: AccountSummary | null;
  lastTestResult: AccountConnectionTestResult | null;
  onFieldChange: (field: keyof AccountFormState, value: string) => void;
  onSave: () => void;
  onTest: () => void;
  onRefresh: () => void;
}

const SECURITY_OPTIONS = [
  { value: "tls", label: "TLS" },
  { value: "start_tls", label: "STARTTLS" },
  { value: "none", label: "None" },
] as const;

export function AccountsWorkspacePanel({
  form,
  accountsState,
  saveState,
  testState,
  saveError,
  testError,
  lastSavedAccount,
  lastTestResult,
  onFieldChange,
  onSave,
  onTest,
  onRefresh,
}: AccountsWorkspacePanelProps) {
  return (
    <div className="workspace-shell__content-switch">
      <div className="workspace-shell__section-head">
        <div>
          <Title3>Accounts onboarding</Title3>
          <Text className="workspace-shell__section-desc">
            这是 M1 的接入首切片：先打通账户模型、CLI / Tauri 契约和连接预检。
          </Text>
        </div>
        <Badge appearance="filled" color="brand">
          非敏感元数据
        </Badge>
      </div>

      <div className="account-panel__hero">
        <Badge appearance="tint">M1 / account add-list-test</Badge>
        <Text>
          当前只保存邮箱、登录名和服务器配置。密码与系统级安全存储会在下一切片接入。
        </Text>
      </div>

      {saveError ? (
        <MessageBar intent="error">
          <MessageBarBody>{saveError}</MessageBarBody>
        </MessageBar>
      ) : null}

      {testError ? (
        <MessageBar intent="error">
          <MessageBarBody>{testError}</MessageBarBody>
        </MessageBar>
      ) : null}

      {lastSavedAccount ? (
        <MessageBar intent="success">
          <MessageBarBody>
            已保存账户 `{lastSavedAccount.display_name}`，当前可继续做连接预检或切换回工作台视图。
          </MessageBarBody>
        </MessageBar>
      ) : null}

      {lastTestResult ? (
        <MessageBar intent={lastTestResult.status === "passed" ? "success" : "warning"}>
          <MessageBarBody>{lastTestResult.summary}</MessageBarBody>
        </MessageBar>
      ) : null}

      <section className="account-panel__section">
        <div className="workspace-shell__section-head">
          <div>
            <div className="workspace-shell__section-title">账户表单</div>
            <Text className="workspace-shell__section-desc">
              先定义手动 IMAP / SMTP 配置，再通过共享服务做连接预检。
            </Text>
          </div>
          <Badge appearance="outline">
            {saveState === "submitting" || testState === "submitting" ? "提交中" : "可编辑"}
          </Badge>
        </div>

        <div className="account-panel__grid">
          <label className="account-panel__field">
            <span className="account-panel__label">账户名称</span>
            <input
              className="account-panel__input"
              value={form.displayName}
              onChange={(event) => onFieldChange("displayName", event.target.value)}
              placeholder="例如 Work Outlook"
            />
          </label>

          <label className="account-panel__field">
            <span className="account-panel__label">邮箱地址</span>
            <input
              className="account-panel__input"
              value={form.email}
              onChange={(event) => onFieldChange("email", event.target.value)}
              placeholder="name@example.com"
            />
          </label>

          <label className="account-panel__field">
            <span className="account-panel__label">登录名</span>
            <input
              className="account-panel__input"
              value={form.login}
              onChange={(event) => onFieldChange("login", event.target.value)}
              placeholder="通常与邮箱相同"
            />
          </label>
        </div>

        <div className="account-panel__server-grid">
          <section className="account-panel__server-card">
            <div className="workspace-shell__section-head">
              <div className="workspace-shell__section-title">IMAP</div>
              <Badge appearance="tint">读取入口</Badge>
            </div>

            <div className="account-panel__grid">
              <label className="account-panel__field">
                <span className="account-panel__label">服务器</span>
                <input
                  className="account-panel__input"
                  value={form.imapHost}
                  onChange={(event) => onFieldChange("imapHost", event.target.value)}
                  placeholder="imap.example.com"
                />
              </label>

              <label className="account-panel__field">
                <span className="account-panel__label">端口</span>
                <input
                  className="account-panel__input"
                  value={form.imapPort}
                  onChange={(event) => onFieldChange("imapPort", event.target.value)}
                  placeholder="993"
                />
              </label>

              <label className="account-panel__field">
                <span className="account-panel__label">安全策略</span>
                <select
                  className="account-panel__select"
                  value={form.imapSecurity}
                  onChange={(event) => onFieldChange("imapSecurity", event.target.value)}
                >
                  {SECURITY_OPTIONS.map((option) => (
                    <option key={option.value} value={option.value}>
                      {option.label}
                    </option>
                  ))}
                </select>
              </label>
            </div>
          </section>

          <section className="account-panel__server-card">
            <div className="workspace-shell__section-head">
              <div className="workspace-shell__section-title">SMTP</div>
              <Badge appearance="tint">提交入口</Badge>
            </div>

            <div className="account-panel__grid">
              <label className="account-panel__field">
                <span className="account-panel__label">服务器</span>
                <input
                  className="account-panel__input"
                  value={form.smtpHost}
                  onChange={(event) => onFieldChange("smtpHost", event.target.value)}
                  placeholder="smtp.example.com"
                />
              </label>

              <label className="account-panel__field">
                <span className="account-panel__label">端口</span>
                <input
                  className="account-panel__input"
                  value={form.smtpPort}
                  onChange={(event) => onFieldChange("smtpPort", event.target.value)}
                  placeholder="587"
                />
              </label>

              <label className="account-panel__field">
                <span className="account-panel__label">安全策略</span>
                <select
                  className="account-panel__select"
                  value={form.smtpSecurity}
                  onChange={(event) => onFieldChange("smtpSecurity", event.target.value)}
                >
                  {SECURITY_OPTIONS.map((option) => (
                    <option key={option.value} value={option.value}>
                      {option.label}
                    </option>
                  ))}
                </select>
              </label>
            </div>
          </section>
        </div>

        <div className="account-panel__actions">
          <Button appearance="primary" onClick={onSave} disabled={saveState === "submitting"}>
            {saveState === "submitting" ? "保存中..." : "保存账户"}
          </Button>
          <Button
            appearance="secondary"
            onClick={onTest}
            disabled={testState === "submitting"}
          >
            {testState === "submitting" ? "预检中..." : "连接预检"}
          </Button>
        </div>
      </section>

      <section className="account-panel__section">
        <div className="workspace-shell__section-head">
          <div>
            <div className="workspace-shell__section-title">已保存账户</div>
            <Text className="workspace-shell__section-desc">
              列表只保存非敏感元数据，便于 CLI 与桌面端共享同一套 onboarding 结果。
            </Text>
          </div>
          <Button appearance="subtle" onClick={onRefresh}>
            重新加载
          </Button>
        </div>

        {accountsState.status === "loading" ? (
          <div className="account-panel__empty">
            <Spinner label="正在读取账户列表..." />
          </div>
        ) : null}

        {accountsState.status === "error" ? (
          <MessageBar intent="warning">
            <MessageBarBody>{accountsState.message}</MessageBarBody>
          </MessageBar>
        ) : null}

        {accountsState.accounts.length === 0 && accountsState.status !== "loading" ? (
          <div className="account-panel__empty">
            <Text>当前还没有账户配置，保存第一条后这里会显示共享元数据清单。</Text>
          </div>
        ) : null}

        {accountsState.accounts.length > 0 ? (
          <div className="account-panel__list">
            {accountsState.accounts.map((account) => (
              <article key={account.id} className="account-panel__list-item">
                <div className="workspace-shell__section-head">
                  <div>
                    <div className="workspace-shell__section-title">{account.display_name}</div>
                    <Text>{account.email}</Text>
                  </div>
                  <Badge appearance="outline">{account.id}</Badge>
                </div>

                <div className="workspace-shell__message-topline">
                  <Badge appearance="filled" color="brand">
                    IMAP {account.imap.port}
                  </Badge>
                  <Badge appearance="outline">{account.imap.host}</Badge>
                  <Badge appearance="outline">{account.imap.security}</Badge>
                </div>

                <div className="workspace-shell__message-topline">
                  <Badge appearance="filled" color="informative">
                    SMTP {account.smtp.port}
                  </Badge>
                  <Badge appearance="outline">{account.smtp.host}</Badge>
                  <Badge appearance="outline">{account.smtp.security}</Badge>
                </div>
              </article>
            ))}
          </div>
        ) : null}
      </section>
    </div>
  );
}
