import { useEffect, useState } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  DismissRegular,
  SquareMultipleRegular,
  SquareRegular,
  SubtractRegular,
} from "@fluentui/react-icons";
import {
  getTitlebarPlatformState,
  getTitlebarWindowAction,
} from "./titlebar-state";
import "./Titlebar.css";

export function Titlebar() {
  const { isMac, isMobile } = getTitlebarPlatformState(navigator.userAgent);
  const [appWindow] = useState(() => (isTauri() ? getCurrentWindow() : null));
  const [isMaximized, setIsMaximized] = useState(false);

  useEffect(() => {
    if (!appWindow) {
      return;
    }

    let disposed = false;
    let unlisten: (() => void) | null = null;

    const syncWindowState = async () => {
      try {
        const nextIsMaximized = await appWindow.isMaximized();

        if (!disposed) {
          setIsMaximized(nextIsMaximized);
        }
      } catch (error) {
        console.error("Failed to sync the maximized window state.", error);
      }
    };

    void syncWindowState();

    void appWindow
      .onResized(() => {
        void syncWindowState();
      })
      .then((detach) => {
        if (disposed) {
          detach();
          return;
        }

        unlisten = detach;
      })
      .catch((error) => {
        console.error("Failed to listen for window resize events.", error);
      });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [appWindow]);

  if (isMobile) {
    return null;
  }

  const windowAction = getTitlebarWindowAction(isMaximized);
  const canControlWindow = appWindow !== null;
  const MaximizeIcon =
    windowAction === "restore" ? SquareMultipleRegular : SquareRegular;

  const handleMinimize = async () => {
    if (!appWindow) {
      return;
    }

    try {
      await appWindow.minimize();
    } catch (error) {
      console.error("Failed to minimize the window.", error);
    }
  };

  const handleToggleMaximize = async () => {
    if (!appWindow) {
      return;
    }

    try {
      await appWindow.toggleMaximize();
      setIsMaximized(await appWindow.isMaximized());
    } catch (error) {
      console.error("Failed to toggle the maximized window state.", error);
    }
  };

  const handleClose = async () => {
    if (!appWindow) {
      return;
    }

    try {
      await appWindow.close();
    } catch (error) {
      console.error("Failed to close the window.", error);
    }
  };

  return (
    <div className={`titlebar ${isMac ? "mac" : "win"}`}>
      <div className="titlebar-drag-region" data-tauri-drag-region>
        {isMac ? null : <div className="titlebar-title">Twill</div>}
      </div>
      {!isMac && (
        <div className="titlebar-controls">
          <button
            aria-label="Minimize window"
            className="titlebar-btn"
            disabled={!canControlWindow}
            type="button"
            onClick={() => {
              void handleMinimize();
            }}
          >
            <SubtractRegular fontSize={14} />
          </button>
          <button
            aria-label={
              windowAction === "restore"
                ? "Restore window size"
                : "Maximize window"
            }
            className="titlebar-btn"
            disabled={!canControlWindow}
            type="button"
            onClick={() => {
              void handleToggleMaximize();
            }}
          >
            <MaximizeIcon fontSize={14} />
          </button>
          <button
            aria-label="Close window"
            className="titlebar-btn close"
            disabled={!canControlWindow}
            type="button"
            onClick={() => {
              void handleClose();
            }}
          >
            <DismissRegular fontSize={14} />
          </button>
        </div>
      )}
    </div>
  );
}
