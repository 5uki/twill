import type { WorkspaceViewId } from "./app-types";

export type WorkspaceCategory =
  | "verifications"
  | "inbox"
  | "sites"
  | "accounts";

export type WorkspaceCategoryGroup = "mail" | "manage";

interface WorkspaceCategoryMeta {
  title: string;
  navigationLabel: string;
  group: WorkspaceCategoryGroup;
  groupLabel: string;
}

const workspaceCategoryMeta: Record<WorkspaceCategory, WorkspaceCategoryMeta> = {
  inbox: {
    title: "收件箱",
    navigationLabel: "收件箱",
    group: "mail",
    groupLabel: "收件箱",
  },
  verifications: {
    title: "验证消息",
    navigationLabel: "验证消息",
    group: "mail",
    groupLabel: "收件箱",
  },
  sites: {
    title: "网站管理",
    navigationLabel: "网站管理",
    group: "manage",
    groupLabel: "管理",
  },
  accounts: {
    title: "账号管理",
    navigationLabel: "账号管理",
    group: "manage",
    groupLabel: "管理",
  },
};

export function mapWorkspaceViewToCategory(
  viewId: WorkspaceViewId,
): WorkspaceCategory {
  switch (viewId) {
    case "recent_verification":
      return "verifications";
    case "all_inbox":
      return "inbox";
    case "site_list":
      return "sites";
    case "accounts":
      return "accounts";
  }
}

export function getWorkspaceTitle(category: WorkspaceCategory): string {
  return workspaceCategoryMeta[category].title;
}

export function getWorkspaceNavigationLabel(
  category: WorkspaceCategory,
): string {
  return workspaceCategoryMeta[category].navigationLabel;
}

export function getWorkspaceGroup(category: WorkspaceCategory): WorkspaceCategoryGroup {
  return workspaceCategoryMeta[category].group;
}

export function getWorkspaceGroupLabel(
  group: WorkspaceCategoryGroup,
): string {
  return group === "mail" ? "收件箱" : "管理";
}
