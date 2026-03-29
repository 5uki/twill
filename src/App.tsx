import { Button, MessageBar, MessageBarBody, Spinner, Text } from "@fluentui/react-components";
import { startTransition, useEffect, useState } from "react";
import "./App.css";
import { WorkspaceShell } from "./features/workspace/WorkspaceShell";
import type { WorkspaceBootstrapSnapshot } from "./features/workspace/model";
import { toWorkspaceViewModel } from "./features/workspace/view-model";
import { loadWorkspaceBootstrap } from "./lib/tauri/loadWorkspaceBootstrap";

type AppState =
  | { status: "loading" }
  | { status: "ready"; snapshot: WorkspaceBootstrapSnapshot }
  | { status: "error"; message: string };

function App() {
  const [state, setState] = useState<AppState>({ status: "loading" });

  async function hydrateWorkspace() {
    setState({ status: "loading" });

    try {
      const snapshot = await loadWorkspaceBootstrap();

      startTransition(() => {
        setState({ status: "ready", snapshot });
      });
    } catch (error) {
      startTransition(() => {
        setState({
          status: "error",
          message: error instanceof Error ? error.message : "工作台启动快照加载失败",
        });
      });
    }
  }

  useEffect(() => {
    void hydrateWorkspace();
  }, []);

  if (state.status === "loading") {
    return (
      <main className="app-state">
        <div className="app-state__card">
          <Spinner label="正在加载工作台底座..." size="large" />
          <Text>当前会从 Tauri command 读取 M0 静态启动快照。</Text>
        </div>
      </main>
    );
  }

  if (state.status === "error") {
    return (
      <main className="app-state">
        <div className="app-state__card app-state__card--error">
          <MessageBar intent="error">
            <MessageBarBody>{state.message}</MessageBarBody>
          </MessageBar>
          <Button appearance="primary" onClick={() => void hydrateWorkspace()}>
            重新加载
          </Button>
        </div>
      </main>
    );
  }

  return <WorkspaceShell viewModel={toWorkspaceViewModel(state.snapshot)} />;
}

export default App;
