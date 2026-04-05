import { describe, expect, test } from "bun:test";
import { MailInboxRegular, PersonAccountsRegular } from "@fluentui/react-icons";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { renderToStaticMarkup } from "react-dom/server";
import type { WorkspaceSiteContextResolution } from "../../src/lib/app-types";
import { AccountsWorkspace } from "../../src/components/AccountsWorkspace";
import { MailWorkspace } from "../../src/components/MailWorkspace";
import { TopHeader } from "../../src/components/TopHeader";
import {
  buildAccountCommandInput,
  createEmptyAccountFormDraft,
} from "../../src/components/account-form";
import { Sidebar } from "../../src/components/Sidebar";

describe("account workspace", () => {
  test("shows the accounts title for the accounts category", () => {
    const markup = renderToStaticMarkup(<MailWorkspace category="accounts" />);

    expect(markup.includes("账号管理")).toBe(true);
    expect(markup.includes("Archive")).toBe(false);
  });

  test("renders inbox rows with a checkbox and unread closed envelope affordance", () => {
    const markup = renderToStaticMarkup(<MailWorkspace category="inbox" />);

    expect(markup.includes("mail-item-checkbox")).toBe(true);
    expect(markup.includes("mail-item-envelope unread")).toBe(true);
    expect(markup.includes("mail-item-star")).toBe(true);
    expect(markup.includes("mail-item-date")).toBe(true);
  });
});

describe("sidebar grouping", () => {
  test("separates mailbox navigation from management navigation", () => {
    const markup = renderToStaticMarkup(
      <Sidebar
        activeCategory="inbox"
        groups={[
          {
            id: "mail",
            label: "收件箱",
            items: [
              { id: "inbox", label: "收件箱", badge: 12, icon: <MailInboxRegular /> },
            ],
          },
          {
            id: "manage",
            label: "管理",
            items: [
              {
                id: "accounts",
                label: "账号管理",
                badge: 3,
                icon: <PersonAccountsRegular />,
              },
            ],
          },
        ]}
        onCategoryChange={() => {}}
      />,
    );

    expect(markup.includes("收件箱")).toBe(true);
    expect(markup.includes("管理")).toBe(true);
    expect(markup.includes("账号管理")).toBe(true);
  });
});

describe("account command input", () => {
  test("keeps login and mail security values aligned with the Rust contract", () => {
    expect(
      buildAccountCommandInput({
        displayName: "Primary Gmail",
        email: "primary@example.com",
        login: "",
        password: "app-password",
        imapHost: "imap.example.com",
        imapPort: "993",
        imapSecurity: "tls",
        smtpHost: "smtp.example.com",
        smtpPort: "587",
        smtpSecurity: "start_tls",
      }),
    ).toEqual({
      display_name: "Primary Gmail",
      email: "primary@example.com",
      login: "primary@example.com",
      password: "app-password",
      imap: {
        host: "imap.example.com",
        port: 993,
        security: "tls",
      },
      smtp: {
        host: "smtp.example.com",
        port: 587,
        security: "start_tls",
      },
    });
  });
});

describe("account workspace copy", () => {
  test("uses user-facing labels instead of developer-facing explanations", () => {
    const markup = renderToStaticMarkup(
      <AccountsWorkspace
        accounts={[]}
        draft={createEmptyAccountFormDraft()}
        errorMessage={null}
        isSaving={false}
        isTesting={false}
        probeResult={null}
        runtimeAvailable
        onDraftChange={() => {}}
        onRefresh={() => {}}
        onSave={() => {}}
        onTest={() => {}}
      />,
    );

    expect(markup.includes("Tauri")).toBe(false);
    expect(markup.includes("CLI")).toBe(false);
    expect(markup.includes("Rust")).toBe(false);
    expect(markup.includes("Display Name")).toBe(false);
    expect(markup.includes("Email Address")).toBe(false);
    expect(markup.includes("App Password")).toBe(false);
    expect(markup.includes("Live Probe")).toBe(false);
    expect(markup.includes("Refresh")).toBe(false);
  });
});

describe("mail row layout", () => {
  test("keeps the date in a dedicated single-line slot and centers the right meta row", () => {
    const source = readFileSync(
      resolve(process.cwd(), "src/components/MailWorkspace.tsx"),
      "utf8",
    );
    const css = readFileSync(resolve(process.cwd(), "src/App.css"), "utf8");

    expect(source.includes('className="mail-item-date"')).toBe(true);
    expect(css.includes(".mail-item-date")).toBe(true);
    expect(css.includes("white-space: nowrap")).toBe(true);
    expect(css.includes("align-items: center")).toBe(true);
  });
});

describe("sync header", () => {
  test("shows user-facing sync summary and action", () => {
    const markup = renderToStaticMarkup(
      <TopHeader
        canSync
        isSyncing={false}
        syncSummary="已同步 1 个账号，共 3 封邮件"
        onSync={() => {}}
      />,
    );

    expect(markup.includes("已同步 1 个账号，共 3 封邮件")).toBe(true);
    expect(markup.includes("立即同步")).toBe(true);
    expect(markup.includes("generated_at")).toBe(false);
  });

  test("shows syncing state while refresh is running", () => {
    const markup = renderToStaticMarkup(
      <TopHeader canSync isSyncing syncSummary="正在刷新最近邮件" onSync={() => {}} />,
    );

    expect(markup.includes("同步中")).toBe(true);
  });
});

describe("current site header", () => {
  test("separates current site input from global search and renders candidate sites", () => {
    const resolution: WorkspaceSiteContextResolution = {
      input: "lin",
      normalized_domain: "lin",
      matched_site: null,
      candidate_sites: [
        {
          id: "site_linear",
          label: "Linear",
          hostname: "linear.app",
          pending_count: 1,
          latest_sender: "hello@linear.app",
        },
      ],
    };
    const markup = renderToStaticMarkup(
      <TopHeader
        canSync
        currentSiteValue="lin"
        searchValue="362149"
        siteResolution={resolution}
        onCurrentSiteChange={() => {}}
        onCurrentSiteSelect={() => {}}
        onSearchChange={() => {}}
        onSync={() => {}}
      />,
    );

    expect(markup.includes("输入当前站点")).toBe(true);
    expect(markup.includes("搜索邮件")).toBe(true);
    expect(markup.includes("linear.app")).toBe(true);
  });
});

describe("site confirmation header", () => {
  test("shows a confirm action when the normalized site is not yet in the site list", () => {
    const resolution: WorkspaceSiteContextResolution = {
      input: "https://vercel.com/login",
      normalized_domain: "vercel.com",
      matched_site: null,
      candidate_sites: [],
    };
    const markup = renderToStaticMarkup(
      <TopHeader
        canSync
        currentSiteValue="https://vercel.com/login"
        searchValue=""
        siteResolution={resolution}
        onCurrentSiteChange={() => {}}
        onCurrentSiteSelect={() => {}}
        onSearchChange={() => {}}
        onSync={() => {}}
      />,
    );

    expect(markup.includes("加入网站清单")).toBe(true);
    expect(markup.includes("vercel.com")).toBe(true);
  });
});

describe("mail workspace reading controls", () => {
  test("shows the recent 48 hours scope, open original action, and read toggle", () => {
    const markup = renderToStaticMarkup(<MailWorkspace category="verifications" />);

    expect(markup.includes("最近 48 小时")).toBe(true);
    expect(markup.includes("查看更早邮件")).toBe(true);
    expect(markup.includes("打开原始邮件")).toBe(true);
    expect(markup.includes("标记已读")).toBe(true);
  });
});
