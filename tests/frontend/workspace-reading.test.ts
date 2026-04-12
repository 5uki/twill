import { describe, expect, test } from "bun:test";
import type {
  WorkspaceBootstrapSnapshot,
  WorkspaceMessageFilter,
} from "../../src/lib/app-types";
import {
  applyWorkspaceMessageAction,
  applyWorkspaceMessageReadState,
  applyWorkspaceMessageStatus,
  buildWorkspaceMessageFilter,
  confirmWorkspaceSite,
  findWorkspaceMessageIdForExtract,
  filterWorkspaceMessages,
  openWorkspaceMessage,
  openWorkspaceMessageOriginal,
  resolveWorkspaceSiteContext,
  resolveSelectedWorkspaceMessage,
} from "../../src/lib/workspace-reading";
import {
  createSampleWorkspaceSnapshot,
  sampleWorkspaceSnapshot,
} from "./workspace-fixture";

const snapshot = sampleWorkspaceSnapshot as WorkspaceBootstrapSnapshot;

describe("workspace reading helpers", () => {
  test("defaults verification filters to the most recent 48 hours", () => {
    const filter = buildWorkspaceMessageFilter("verifications", "all", "");

    expect(filter.recent_hours).toBe(48);
  });

  test("supports category and query filtering for reading flow", () => {
    const filter: WorkspaceMessageFilter = {
      category: "security",
      query: "362149",
    };

    const messages = filterWorkspaceMessages(snapshot, filter);

    expect(messages).toHaveLength(1);
    expect(messages[0]?.id).toBe("msg_github_security");
  });

  test("supports exact site filtering for reading flow", () => {
    const filter: WorkspaceMessageFilter = {
      site_hint: "github.com",
    };

    const messages = filterWorkspaceMessages(snapshot, filter);

    expect(messages).toHaveLength(1);
    expect(messages[0]?.id).toBe("msg_github_security");
  });

  test("falls back to the first visible detail when preferred message is filtered out", () => {
    const messages = filterWorkspaceMessages(snapshot, {
      verification_only: true,
      query: "linear",
    });

    const selected = resolveSelectedWorkspaceMessage(
      snapshot,
      messages,
      "msg_github_security",
    );

    expect(messages).toHaveLength(1);
    expect(selected?.id).toBe("msg_linear_verify");
  });

  test("updates local snapshot when a message is marked as processed", () => {
    const nextSnapshot = applyWorkspaceMessageStatus(
      snapshot,
      "msg_github_security",
      "processed",
    );
    const githubSite = nextSnapshot.site_summaries.find(
      (site) => site.hostname === "github.com",
    );
    const pendingGroup = nextSnapshot.message_groups.find(
      (group) => group.id === "pending",
    );
    const processedGroup = nextSnapshot.message_groups.find(
      (group) => group.id === "processed",
    );

    expect(nextSnapshot.selected_message.status).toBe("processed");
    expect(
      nextSnapshot.message_details.find((detail) => detail.id === "msg_github_security")
        ?.status,
    ).toBe("processed");
    expect(pendingGroup?.items.some((item) => item.id === "msg_github_security")).toBe(
      false,
    );
    expect(
      processedGroup?.items.some((item) => item.id === "msg_github_security"),
    ).toBe(true);
    expect(githubSite?.pending_count).toBe(0);
  });

  test("updates local snapshot when a message is marked read without changing status", () => {
    const nextSnapshot = applyWorkspaceMessageReadState(
      snapshot,
      "msg_github_security",
      "read",
    );
    const inbox = nextSnapshot.mailboxes.find(
      (mailbox) => mailbox.id === "acct_primary-example-com/inbox",
    );

    expect(nextSnapshot.selected_message.read_state).toBe("read");
    expect(nextSnapshot.selected_message.status).toBe("pending");
    expect(
      nextSnapshot.message_details.find((detail) => detail.id === "msg_github_security")
        ?.read_state,
    ).toBe("read");
    expect(inbox?.unread_count).toBe(1);
  });

  test("resolves current site from a pasted url and returns candidates when not exact", () => {
    const exact = resolveWorkspaceSiteContext(
      snapshot,
      "https://www.github.com/login/device",
    );
    const candidateOnly = resolveWorkspaceSiteContext(snapshot, "lin");

    expect(exact.normalized_domain).toBe("github.com");
    expect(exact.matched_site?.hostname).toBe("github.com");
    expect(candidateOnly.matched_site).toBeNull();
    expect(candidateOnly.candidate_sites[0]?.hostname).toBe("linear.app");
  });

  test("applies high-value message action by marking processed and removing matching extract", () => {
    const result = applyWorkspaceMessageAction(
      snapshot,
      "msg_github_security",
      "copy_code",
    );

    expect(result.action).toBe("copy_code");
    expect(result.copied_value).toBe("362149");
    expect(result.snapshot.selected_message.status).toBe("processed");
    expect(
      result.snapshot.extracts.some((extract) => extract.id === "extract_github_code"),
    ).toBe(false);
  });

  test("finds the backing message id for an extract action", () => {
    const extract = snapshot.extracts.find((item) => item.id === "extract_linear_link");

    expect(extract).toBeDefined();
    expect(findWorkspaceMessageIdForExtract(snapshot, extract!)).toBe(
      "msg_linear_verify",
    );
  });

  test("filters verification view to the most recent 48 hours by default", () => {
    const datedSnapshot = createSampleWorkspaceSnapshot();

    datedSnapshot.generated_at = "2026-04-05T09:00:00Z";
    datedSnapshot.message_groups[0]!.items[1]!.received_at = "2026-04-01T08:41:00Z";
    datedSnapshot.message_details[1]!.received_at = "2026-04-01T08:41:00Z";

    const messages = filterWorkspaceMessages(datedSnapshot, {
      verification_only: true,
      recent_hours: 48,
    });

    expect(messages).toHaveLength(1);
    expect(messages[0]?.id).toBe("msg_github_security");
  });

  test("opens a message by marking it read without changing processed status", () => {
    const result = openWorkspaceMessage(snapshot, "msg_github_security");
    const inbox = result.snapshot.mailboxes.find(
      (mailbox) => mailbox.id === "acct_primary-example-com/inbox",
    );

    expect(result.detail.id).toBe("msg_github_security");
    expect(result.snapshot.selected_message.read_state).toBe("read");
    expect(result.snapshot.selected_message.status).toBe("pending");
    expect(inbox?.unread_count).toBe(1);
  });

  test("opens the original message by returning the original url and marking it read", () => {
    const result = openWorkspaceMessageOriginal(snapshot, "msg_linear_verify");

    expect(result.original_url).toContain("msg_linear_verify");
    expect(
      result.snapshot.message_details.find((detail) => detail.id === "msg_linear_verify")
        ?.read_state,
    ).toBe("read");
  });

  test("confirms a manually entered site into the site list", () => {
    const nextSnapshot = confirmWorkspaceSite(snapshot, "vercel.com");
    const resolution = resolveWorkspaceSiteContext(nextSnapshot, "https://vercel.com/login");

    expect(
      nextSnapshot.site_summaries.some((site) => site.hostname === "vercel.com"),
    ).toBe(true);
    expect(resolution.matched_site?.hostname).toBe("vercel.com");
  });
});
