# Twill Claude Code Instructions

## Session Start

- Start new development sessions with `/trellis:start` by default.
- If the session did not begin with `/trellis:start`, do not start the task immediately.
- First ask the user whether they explicitly want to continue without `/trellis:start`.
- If the user chooses to skip it, still do the minimum context read:
  - `.trellis/workflow.md`
  - relevant `.trellis/spec/*/index.md`
  - current task / scope / local context

## Project Context

- Twill is a multi-platform email client app built with `Tauri + React + TypeScript + Rust`.
- The UI framework is explicitly `React`.
- `src/` is the React UI layer.
- `src-tauri/` is the Tauri / Rust native layer.

## Hard Rules

- Think, explain, and respond in simplified Chinese.
- Write comments and project documentation in Chinese by default.
- Follow strict TDD: `Red -> Green -> Refactor`.
- Every feature must have a CLI simulator.
- A feature without a CLI simulator is considered incomplete.
- CLI, Tauri commands, and tests must share the same core application logic.
- Do not preserve backward-compatibility branches unless the requirement explicitly asks for them.
- Avoid over-defensive programming.
- Keep code simple, clear, and maintainable.

## Quality Focus

- Check time complexity and space complexity.
- Pay attention to memory usage, IO cost, and hot paths.
- Cover success paths, failure paths, and edge cases.
- For cross-layer changes, verify command contracts, type mapping, and CLI coverage.

## Output Style

- Optimize responses for terminal readability.
- Lead with the core conclusion on complex topics.
- Use clear sections and short lists.
- Prefer concise ASCII diagrams for complex flows.
- Use fenced code blocks for multi-line code, config, or logs.
