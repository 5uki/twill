import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import {
  getTitlebarPlatformState,
  getTitlebarWindowAction,
} from "../../src/components/titlebar-state";

describe("titlebar state helpers", () => {
  test("shows the maximize action when the window is not maximized", () => {
    expect(getTitlebarWindowAction(false)).toBe("maximize");
  });

  test("shows the restore action when the window is maximized", () => {
    expect(getTitlebarWindowAction(true)).toBe("restore");
  });

  test("treats macOS desktop as mac but not mobile", () => {
    expect(
      getTitlebarPlatformState(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15",
      ),
    ).toEqual({
      isMac: true,
      isMobile: false,
    });
  });

  test("treats iPhone user agents as mobile instead of macOS desktop", () => {
    expect(
      getTitlebarPlatformState(
        "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15",
      ),
    ).toEqual({
      isMac: false,
      isMobile: true,
    });
  });
});

describe("custom titlebar capability", () => {
  test("grants the required window permissions", () => {
    const capabilityPath = resolve(
      process.cwd(),
      "src-tauri/capabilities/default.json",
    );
    const capability = JSON.parse(readFileSync(capabilityPath, "utf8")) as {
      permissions: string[];
    };

    expect(capability.permissions.includes("core:window:allow-close")).toBe(
      true,
    );
    expect(capability.permissions.includes("core:window:allow-minimize")).toBe(
      true,
    );
    expect(
      capability.permissions.includes("core:window:allow-toggle-maximize"),
    ).toBe(true);
    expect(
      capability.permissions.includes("core:window:allow-start-dragging"),
    ).toBe(true);
  });
});
