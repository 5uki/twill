# 后端开发规范

> Twill 的 backend 指 `src-tauri/` 中的 Tauri / Rust 原生层，以及与之对应的命令、CLI 和测试入口。

---

## 必读

> `CLI 模拟器` 是本目录最高优先级约束之一。没有 CLI 模拟器的功能，默认不算完成。

开始任何 Tauri / Rust / CLI 相关任务前，先阅读：

1. `../guides/project-context.md`
2. `../guides/project-engineering-baseline.md`
3. `../guides/session-rules.md`

---

## 指南索引

| 指南 | 说明 | 状态 |
|------|------|------|
| [Directory Structure](./directory-structure.md) | `src-tauri/` 目录职责与演进方向 | Ready |
| [Tauri Command Guidelines](./tauri-command-guidelines.md) | 命令契约、CLI 模拟器、跨层边界 | Ready |
| [Account & Workspace Contracts](./account-workspace-contracts.md) | 账户接入、实时探测、工作台快照的实际合同 | Ready |
| [Quality Guidelines](./quality-guidelines.md) | TDD、性能、代码整洁与验证要求 | Ready |
| [Type Safety](./type-safety.md) | Rust 类型边界、序列化与错误模型 | Ready |

---

## 开发前检查

- [ ] 已确认这是 `src-tauri/` 任务还是跨层任务
- [ ] 已明确命令输入 / 输出模型
- [ ] 已规划对应 CLI 模拟器和测试
- [ ] 已明确 CLI 要覆盖哪些成功、失败和边界场景
- [ ] 已识别需要同步修改的前端调用层
- [ ] 如果涉及基础设施或跨层合同，已同步更新相关 code-spec

---

## 语言要求

本目录规范默认使用中文维护。
