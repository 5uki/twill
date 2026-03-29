import { invoke } from "@tauri-apps/api/core";
import type { WorkspaceBootstrapSnapshot } from "../../features/workspace/model";

export function loadWorkspaceBootstrap() {
  return invoke<WorkspaceBootstrapSnapshot>("load_workspace_bootstrap");
}
