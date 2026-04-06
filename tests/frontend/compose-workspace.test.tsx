import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { ComposeWorkspace } from "../../src/components/ComposeWorkspace";
import {
  buildSendMessageCommandInput,
  createEmptyComposeFormDraft,
  prepareComposeDraftFromMessage,
} from "../../src/components/compose-form";

const sampleAccounts = [
  {
    id: "acct_primary-example-com",
    display_name: "Primary Gmail",
    email: "primary@example.com",
    login: "primary@example.com",
    credential_state: "stored" as const,
    imap: {
      host: "imap.example.com",
      port: 993,
      security: "tls" as const,
    },
    smtp: {
      host: "smtp.example.com",
      port: 587,
      security: "start_tls" as const,
    },
  },
];

const sampleMessage = {
  id: "msg_github_security",
  account_id: "acct_primary-example-com",
  subject: "GitHub 安全验证码",
  sender: "noreply@github.com",
  account_name: "Primary Gmail",
  mailbox_id: "mailbox_primary_inbox",
  mailbox_label: "Inbox",
  received_at: "2026-04-05T08:58:00Z",
  category: "security" as const,
  status: "pending" as const,
  read_state: "unread" as const,
  site_hint: "github.com",
  summary: "GitHub 安全验证码 362149",
  extracted_code: "362149",
  verification_link: "https://github.com/login",
  original_message_url: "https://mail.example.com/messages/msg_github_security",
  body_text: "GitHub 安全验证码\n362149",
  prefetched_body: true,
  synced_at: "2026-04-05T09:00:00Z",
};

describe("compose workspace", () => {
  test("renders compose fields and structured send feedback", () => {
    const markup = renderToStaticMarkup(
      <ComposeWorkspace
        accounts={sampleAccounts}
        draft={{
          accountId: "acct_primary-example-com",
          to: "dev@example.com",
          subject: "Launch update",
          body: "Shipping today.",
        }}
        errorMessage={null}
        isSending={false}
        mode="new"
        result={{
          account_id: "acct_primary-example-com",
          to: "dev@example.com",
          subject: "Launch update",
          status: "sent",
          delivery_mode: "simulated",
          summary: "模拟发送已提交",
          smtp_endpoint: "smtp.example.com:587",
        }}
        runtimeAvailable
        onDraftChange={() => {}}
        onResetToNew={() => {}}
        onSend={() => {}}
        sourceMessage={null}
      />,
    );

    expect(markup.includes("新建邮件")).toBe(true);
    expect(markup.includes("收件人")).toBe(true);
    expect(markup.includes("邮件主题")).toBe(true);
    expect(markup.includes("发送邮件")).toBe(true);
    expect(markup.includes("模拟提交")).toBe(true);
    expect(markup.includes("smtp.example.com:587")).toBe(true);
  });

  test("keeps the command input aligned with the Rust send contract", () => {
    expect(
      buildSendMessageCommandInput({
        accountId: "  acct_primary-example-com ",
        to: " dev@example.com ",
        subject: " Launch update ",
        body: " Shipping today. ",
      }),
    ).toEqual({
      account_id: "acct_primary-example-com",
      to: "dev@example.com",
      subject: "Launch update",
      body: "Shipping today.",
    });
  });

  test("shows desktop-only guidance in browser preview mode", () => {
    const markup = renderToStaticMarkup(
      <ComposeWorkspace
        accounts={sampleAccounts}
        draft={createEmptyComposeFormDraft("acct_primary-example-com")}
        errorMessage={null}
        isSending={false}
        mode="new"
        result={null}
        runtimeAvailable={false}
        onDraftChange={() => {}}
        onResetToNew={() => {}}
        onSend={() => {}}
        sourceMessage={null}
      />,
    );

    expect(markup.includes("浏览器预览不支持真实发信")).toBe(true);
  });

  test("renders reply mode source message context", () => {
    const markup = renderToStaticMarkup(
      <ComposeWorkspace
        accounts={sampleAccounts}
        draft={{
          accountId: "acct_primary-example-com",
          to: "noreply@github.com",
          subject: "Re: GitHub 安全验证码",
          body: "\n\n在 2026-04-05T08:58:00Z，noreply@github.com 写道：\n> GitHub 安全验证码",
        }}
        errorMessage={null}
        isSending={false}
        mode="reply"
        result={null}
        runtimeAvailable
        onDraftChange={() => {}}
        onResetToNew={() => {}}
        onSend={() => {}}
        sourceMessage={sampleMessage}
      />,
    );

    expect(markup.includes("回复邮件")).toBe(true);
    expect(markup.includes("来源邮件")).toBe(true);
    expect(markup.includes("GitHub 安全验证码")).toBe(true);
    expect(markup.includes("切回新建")).toBe(true);
  });

  test("builds reply and forward fallback drafts from a workspace message", () => {
    expect(prepareComposeDraftFromMessage("reply", sampleMessage)).toEqual({
      mode: "reply",
      account_id: "acct_primary-example-com",
      to: "noreply@github.com",
      subject: "Re: GitHub 安全验证码",
      body: "\n\n在 2026-04-05T08:58:00Z，noreply@github.com 写道：\n> GitHub 安全验证码\n> 362149",
      source_message_id: "msg_github_security",
    });

    expect(
      prepareComposeDraftFromMessage("forward", {
        ...sampleMessage,
        subject: "Fwd: GitHub 安全验证码",
      }),
    ).toEqual({
      mode: "forward",
      account_id: "acct_primary-example-com",
      to: "",
      subject: "Fwd: GitHub 安全验证码",
      body: [
        "",
        "",
        "---------- 转发邮件 ----------",
        "发件人: noreply@github.com",
        "账号: Primary Gmail",
        "时间: 2026-04-05T08:58:00Z",
        "主题: Fwd: GitHub 安全验证码",
        "",
        "GitHub 安全验证码\n362149",
      ].join("\n"),
      source_message_id: "msg_github_security",
    });
  });
});
