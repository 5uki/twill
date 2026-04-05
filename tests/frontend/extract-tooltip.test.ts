import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { getCircularAvatarMetrics } from "../../src/components/extract-geometry";
import { getExtractActionHint } from "../../src/components/extract-tooltip";

describe("extract tooltip copy", () => {
  test("shows a copy hint for verification codes", () => {
    expect(getExtractActionHint("code", false)).toBe("Copy code");
  });

  test("shows an open hint for verification links", () => {
    expect(getExtractActionHint("link", false)).toBe("Open link");
  });

  test("hides the tooltip after copying a verification code", () => {
    expect(getExtractActionHint("code", true)).toBeNull();
  });
});

describe("extract card tooltip implementation", () => {
  test("uses a top-level portal tooltip instead of browser or library tooltip wrappers", () => {
    const source = readFileSync(
      resolve(process.cwd(), "src/components/MailWorkspace.tsx"),
      "utf8",
    );
    const portalSource = readFileSync(
      resolve(process.cwd(), "src/components/ExtractTooltipPortal.tsx"),
      "utf8",
    );

    expect(
      source.includes("import { Text, Avatar, Tooltip } from '@fluentui/react-components';"),
    ).toBe(false);
    expect(source.includes("title=")).toBe(false);
    expect(portalSource.includes("createPortal")).toBe(true);
  });
});

describe("circular avatar geometry", () => {
  test("uses centered avatar metrics with even inset around the progress ring", () => {
    expect(getCircularAvatarMetrics()).toEqual({
      center: 16,
      innerSize: 24,
      outerSize: 32,
      radius: 13,
      strokeWidth: 2,
    });
  });
});

describe("header and shell configuration", () => {
  test("uses a full-width search header without an account avatar", () => {
    const headerSource = readFileSync(
      resolve(process.cwd(), "src/components/TopHeader.tsx"),
      "utf8",
    );

    expect(headerSource.includes("Avatar")).toBe(false);
    expect(headerSource.includes("className=\"top-header-search\"")).toBe(true);
  });

  test("uses a relative Vite base so Tauri webview refresh does not blank the app", () => {
    const viteConfigSource = readFileSync(
      resolve(process.cwd(), "vite.config.ts"),
      "utf8",
    );

    expect(viteConfigSource.includes('base: "./"')).toBe(true);
  });

  test("uses focus styling instead of hover styling for the dismiss button", () => {
    const appCssSource = readFileSync(
      resolve(process.cwd(), "src/App.css"),
      "utf8",
    );

    expect(appCssSource.includes(".extract-minimal-close:hover")).toBe(false);
    expect(appCssSource.includes(".extract-minimal-close:focus-visible")).toBe(
      true,
    );
    expect(appCssSource.includes(".extract-minimal-code.link")).toBe(true);
    expect(appCssSource.includes("#2563eb")).toBe(true);
  });
});
