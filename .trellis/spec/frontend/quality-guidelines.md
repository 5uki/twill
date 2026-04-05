# Quality Guidelines

> Code quality standards for frontend development.

---

## Overview

<!--
Document your project's quality standards here.

Questions to answer:
- What patterns are forbidden?
- What linting rules do you enforce?
- What are your testing requirements?
- What code review standards apply?
-->

(To be filled by the team)

---

## Forbidden Patterns

<!-- Patterns that should never be used and why -->

(To be filled by the team)

---

## Required Patterns

### Desktop Shell Invariants

- Tauri desktop builds must keep `base: "./"` in `vite.config.ts`.
- If `base` falls back to `/`, bundled assets resolve against the filesystem root after a WebView refresh and the app can render a blank screen.
- Custom overlays inside scrollable desktop surfaces should render through a portal to `document.body`.
- Do not use browser-native `title` tooltips for extract pills, titlebar controls, or other dense desktop interactions.

---

## Testing Requirements

- Add a focused Bun test for each non-obvious desktop shell regression.
- For Tauri shell fixes, prefer assertions on config and helper contracts in addition to normal UI build/test coverage.

---

## Code Review Checklist

<!-- What reviewers should check -->

(To be filled by the team)
