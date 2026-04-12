import {
  Badge,
  Button,
  Field,
  Input,
  Select,
  Spinner,
  Text,
  Textarea,
} from "@fluentui/react-components";
import type {
  AccountSummary,
  ComposeMode,
  SendMessageResult,
  WorkspaceMessageDetail,
} from "../lib/app-types";
import type { ComposeFormDraft } from "./compose-form";

interface ComposeWorkspaceProps {
  accounts: AccountSummary[];
  draft: ComposeFormDraft;
  mode: ComposeMode;
  sourceMessage: WorkspaceMessageDetail | null;
  isSending: boolean;
  errorMessage: string | null;
  result: SendMessageResult | null;
  runtimeAvailable: boolean;
  onDraftChange: (field: keyof ComposeFormDraft, value: string) => void;
  onSend: () => void;
  onResetToNew?: () => void;
}

export function ComposeWorkspace({
  accounts,
  draft,
  mode,
  sourceMessage,
  isSending,
  errorMessage,
  result,
  runtimeAvailable,
  onDraftChange,
  onSend,
  onResetToNew,
}: ComposeWorkspaceProps) {
  const canSend = runtimeAvailable && accounts.length > 0 && !isSending;
  const modeLabel =
    mode === "reply" ? "回复" : mode === "forward" ? "转发" : "新建";

  return (
    <div className="workspace-content">
      <div className="workspace-title-row">
        <div className="workspace-title-stack">
          <Text style={{ fontWeight: 600, fontSize: "18px" }}>{modeLabel}邮件</Text>
          <Text className="workspace-subtitle">
            选择账号、填写收件人和正文，并获得结构化发送反馈。
          </Text>
        </div>
        <div className="compose-header-badges">
          <Badge appearance="tint">模式 {modeLabel}</Badge>
          <Badge appearance="outline">账号 {accounts.length}</Badge>
        </div>
      </div>

      <section className="accounts-panel compose-panel">
        <div className="accounts-panel-header">
          <div>
            <Text className="accounts-panel-title">发送设置</Text>
            <Text className="accounts-panel-subtitle">
              {runtimeAvailable
                ? "桌面端会复用已保存的 SMTP 配置与系统密码执行发送。"
                : "浏览器预览不支持真实发信，只用于查看界面结构。"}
            </Text>
          </div>
          {mode !== "new" ? (
            <Button appearance="subtle" disabled={isSending} onClick={onResetToNew}>
              切回新建
            </Button>
          ) : null}
        </div>

        {sourceMessage ? (
          <div className="compose-source-card">
            <div className="compose-source-row">
              <Text className="compose-result-label">来源邮件</Text>
              <Badge appearance="outline">{modeLabel}</Badge>
            </div>
            <Text className="accounts-panel-title">{sourceMessage.subject}</Text>
            <Text className="accounts-panel-subtitle">
              {sourceMessage.sender} · {sourceMessage.account_name} · {sourceMessage.mailbox_label}
            </Text>
          </div>
        ) : null}

        {accounts.length === 0 ? (
          <div className="accounts-empty">
            <Text>
              {runtimeAvailable
                ? "请先到“账号管理”中添加至少一个账号，再回来发送邮件。"
                : "浏览器预览不会加载真实账号列表，这里只展示写信界面和预填结果。"}
            </Text>
          </div>
        ) : null}

        <div className="compose-grid">
          <Field label="发件账号">
            <Select
              disabled={!runtimeAvailable || accounts.length === 0 || isSending}
              value={draft.accountId}
              onChange={(event) =>
                onDraftChange("accountId", event.currentTarget.value)
              }
            >
              {accounts.map((account) => (
                <option key={account.id} value={account.id}>
                  {account.display_name} ({account.email})
                </option>
              ))}
            </Select>
          </Field>
          <Field label="收件人">
            <Input
              disabled={!runtimeAvailable || isSending}
              placeholder="dev@example.com"
              value={draft.to}
              onChange={(_, data) => onDraftChange("to", data.value)}
            />
          </Field>
        </div>

        <Field label="邮件主题">
          <Input
            disabled={!runtimeAvailable || isSending}
            placeholder="输入这封邮件的主题"
            value={draft.subject}
            onChange={(_, data) => onDraftChange("subject", data.value)}
          />
        </Field>

        <Field label="正文">
          <Textarea
            className="compose-body-input"
            disabled={!runtimeAvailable || isSending}
            placeholder="写下你要发送的内容"
            resize="vertical"
            value={draft.body}
            onChange={(_, data) => onDraftChange("body", data.value)}
          />
        </Field>

        <div className="accounts-actions">
          <Button appearance="primary" disabled={!canSend} onClick={onSend}>
            {isSending ? "发送中..." : "发送邮件"}
          </Button>
          {isSending ? <Spinner size="tiny" /> : null}
        </div>

        {errorMessage ? (
          <div className="accounts-error">
            <Text>{errorMessage}</Text>
          </div>
        ) : null}

        {result ? (
          <div className="compose-result">
            <div className="compose-result-header">
              <Text className="accounts-panel-title">发送反馈</Text>
              <Badge appearance="outline">
                {result.delivery_mode === "simulated" ? "模拟提交" : result.delivery_mode}
              </Badge>
            </div>
            <Text className="accounts-panel-subtitle">{result.summary}</Text>
            <div className="compose-result-grid">
              <div className="compose-result-item">
                <Text className="compose-result-label">账号</Text>
                <Text>{result.account_id}</Text>
              </div>
              <div className="compose-result-item">
                <Text className="compose-result-label">收件人</Text>
                <Text>{result.to}</Text>
              </div>
              <div className="compose-result-item">
                <Text className="compose-result-label">主题</Text>
                <Text>{result.subject}</Text>
              </div>
              <div className="compose-result-item">
                <Text className="compose-result-label">SMTP</Text>
                <Text>{result.smtp_endpoint}</Text>
              </div>
            </div>
          </div>
        ) : null}
      </section>
    </div>
  );
}
