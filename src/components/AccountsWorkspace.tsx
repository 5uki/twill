import {
  Badge,
  Button,
  Field,
  Input,
  Select,
  Spinner,
  Text,
} from "@fluentui/react-components";
import type {
  AccountConnectionTestResult,
  AccountSummary,
  MailSecurity,
} from "../lib/app-types";
import type { AccountFormDraft } from "./account-form";

interface AccountsWorkspaceProps {
  accounts: AccountSummary[];
  draft: AccountFormDraft;
  isSaving: boolean;
  isTesting: boolean;
  errorMessage: string | null;
  probeResult: AccountConnectionTestResult | null;
  runtimeAvailable: boolean;
  onDraftChange: (field: keyof AccountFormDraft, value: string) => void;
  onSave: () => void;
  onTest: () => void;
  onRefresh: () => void;
}

const securityOptions: Array<{ label: string; value: MailSecurity }> = [
  { label: "TLS / SSL", value: "tls" },
  { label: "STARTTLS", value: "start_tls" },
  { label: "无", value: "none" },
];

export function AccountsWorkspace({
  accounts,
  draft,
  isSaving,
  isTesting,
  errorMessage,
  probeResult,
  runtimeAvailable,
  onDraftChange,
  onSave,
  onTest,
  onRefresh,
}: AccountsWorkspaceProps) {
  return (
    <div className="accounts-workspace">
      <section className="accounts-panel">
        <div className="accounts-panel-header">
          <div>
            <Text className="accounts-panel-title">已添加账号</Text>
          </div>
          <Button
            appearance="subtle"
            disabled={!runtimeAvailable || isSaving || isTesting}
            onClick={onRefresh}
          >
            刷新
          </Button>
        </div>

        {accounts.length === 0 ? (
          <div className="accounts-empty">
            <Text>
              {runtimeAvailable
                ? "还没有添加任何账号。"
                : "当前环境不支持保存账号，请在桌面端使用。"}
            </Text>
          </div>
        ) : (
          <div className="accounts-list">
            {accounts.map((account) => (
              <div className="accounts-list-item" key={account.id}>
                <div>
                  <Text className="accounts-list-title">
                    {account.display_name}
                  </Text>
                  <Text className="accounts-list-subtitle">
                    {account.email}
                    {account.login !== account.email
                      ? ` · 登录账号 ${account.login}`
                      : ""}
                  </Text>
                </div>
                <div className="accounts-list-badges">
                  <Badge appearance="tint">
                    {account.credential_state === "stored"
                      ? "密码已保存"
                      : "缺少密码"}
                  </Badge>
                  <Badge appearance="outline">
                    IMAP {account.imap.host}:{account.imap.port}
                  </Badge>
                  <Badge appearance="outline">
                    SMTP {account.smtp.host}:{account.smtp.port}
                  </Badge>
                </div>
              </div>
            ))}
          </div>
        )}
      </section>

      <section className="accounts-panel">
        <div className="accounts-panel-header">
          <div>
            <Text className="accounts-panel-title">添加账号</Text>
          </div>
        </div>

        <div className="accounts-grid">
          <Field label="显示名称">
            <Input
              value={draft.displayName}
              onChange={(_, data) => onDraftChange("displayName", data.value)}
            />
          </Field>
          <Field label="邮箱地址">
            <Input
              value={draft.email}
              onChange={(_, data) => onDraftChange("email", data.value)}
            />
          </Field>
          <Field label="登录账号">
            <Input
              placeholder="留空则默认使用邮箱地址"
              value={draft.login}
              onChange={(_, data) => onDraftChange("login", data.value)}
            />
          </Field>
          <Field label="邮箱密码">
            <Input
              type="password"
              value={draft.password}
              onChange={(_, data) => onDraftChange("password", data.value)}
            />
          </Field>
        </div>

        <div className="accounts-server-grid">
          <MailServerSection
            hostField="imapHost"
            hostValue={draft.imapHost}
            portField="imapPort"
            portValue={draft.imapPort}
            securityField="imapSecurity"
            securityValue={draft.imapSecurity}
            title="IMAP"
            onDraftChange={onDraftChange}
          />
          <MailServerSection
            hostField="smtpHost"
            hostValue={draft.smtpHost}
            portField="smtpPort"
            portValue={draft.smtpPort}
            securityField="smtpSecurity"
            securityValue={draft.smtpSecurity}
            title="SMTP"
            onDraftChange={onDraftChange}
          />
        </div>

        <div className="accounts-actions">
          <Button
            appearance="primary"
            disabled={!runtimeAvailable || isSaving || isTesting}
            onClick={onSave}
          >
            {isSaving ? "保存中..." : "保存账号"}
          </Button>
          <Button
            appearance="secondary"
            disabled={!runtimeAvailable || isSaving || isTesting}
            onClick={onTest}
          >
            {isTesting ? "测试中..." : "测试连接"}
          </Button>
          {(isSaving || isTesting) && <Spinner size="tiny" />}
        </div>

        {errorMessage ? (
          <div className="accounts-error">
            <Text>{errorMessage}</Text>
          </div>
        ) : null}

        {probeResult ? (
          <div className="accounts-probe">
            <Text className="accounts-panel-title">
              连接测试{probeResult.status === "passed" ? "通过" : "失败"}
            </Text>
            <Text className="accounts-panel-subtitle">{probeResult.summary}</Text>
            <div className="accounts-probe-checks">
              {probeResult.checks.map((check) => (
                <div className="accounts-probe-check" key={check.target}>
                  <Badge
                    appearance={
                      check.status === "passed" ? "filled" : "outline"
                    }
                    color={check.status === "passed" ? "success" : "danger"}
                  >
                    {check.target.toUpperCase()}
                  </Badge>
                  <Text>{check.message}</Text>
                </div>
              ))}
            </div>
          </div>
        ) : null}
      </section>
    </div>
  );
}

interface MailServerSectionProps {
  title: string;
  hostField: keyof AccountFormDraft;
  hostValue: string;
  portField: keyof AccountFormDraft;
  portValue: string;
  securityField: keyof AccountFormDraft;
  securityValue: MailSecurity;
  onDraftChange: (field: keyof AccountFormDraft, value: string) => void;
}

function MailServerSection({
  title,
  hostField,
  hostValue,
  portField,
  portValue,
  securityField,
  securityValue,
  onDraftChange,
}: MailServerSectionProps) {
  return (
    <div className="accounts-server-card">
      <Text className="accounts-panel-title">{title}</Text>
      <Field label="服务器">
        <Input
          className="font-mono"
          value={hostValue}
          onChange={(_, data) => onDraftChange(hostField, data.value)}
        />
      </Field>
      <div className="accounts-server-row">
        <Field label="端口">
          <Input
            className="font-mono"
            value={portValue}
            onChange={(_, data) => onDraftChange(portField, data.value)}
          />
        </Field>
        <Field label="加密方式">
          <Select
            value={securityValue}
            onChange={(event) =>
              onDraftChange(securityField, event.currentTarget.value)
            }
          >
            {securityOptions.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </Select>
        </Field>
      </div>
    </div>
  );
}
