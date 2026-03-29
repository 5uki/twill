import { describe, expect, test } from "bun:test";
import type { WorkspaceBootstrapSnapshot } from "../../src/features/workspace/model";
import { toWorkspaceViewModel } from "../../src/features/workspace/view-model";

const snapshot: WorkspaceBootstrapSnapshot = {
  app_name: "Twill",
  generated_at: "2026-03-29T09:00:00Z",
  default_view: "recent_verification",
  navigation: [
    { id: "recent_verification", label: "Recent verification", badge: 12 },
    { id: "all_inbox", label: "All inbox", badge: 128 },
  ],
  message_groups: [
    {
      id: "pending",
      label: "待处理",
      items: [
        {
          id: "msg-1",
          subject: "GitHub 安全验证码",
          sender: "noreply@github.com",
          account_name: "Primary Gmail",
          received_at: "2026-03-29T08:58:00Z",
          category: "security",
          status: "pending",
          has_code: true,
          has_link: false,
          preview: "你的 GitHub 登录验证码是 362149。",
        },
      ],
    },
  ],
  selected_message: {
    id: "msg-1",
    subject: "GitHub 安全验证码",
    sender: "noreply@github.com",
    account_name: "Primary Gmail",
    received_at: "2026-03-29T08:58:00Z",
    category: "security",
    status: "pending",
    site_hint: "github.com",
    summary: "这是一个安全验证码邮件。",
    extracted_code: "362149",
    verification_link: "https://github.com/login/device",
  },
};

describe("toWorkspaceViewModel", () => {
  test("marks the default navigation item as active", () => {
    const viewModel = toWorkspaceViewModel(snapshot);

    expect(viewModel.title).toBe("Recent verification");
    expect(viewModel.navigation[0]?.isActive).toBe(true);
    expect(viewModel.navigation[1]?.isActive).toBe(false);
  });

  test("builds enabled primary actions from extracted signals", () => {
    const viewModel = toWorkspaceViewModel(snapshot);

    expect(viewModel.detail.actions[0]).toEqual({
      id: "copy_code",
      label: "复制验证码",
      appearance: "primary",
      body: "362149",
      disabled: false,
    });
    expect(viewModel.detail.actions[1]?.disabled).toBe(false);
  });
});
