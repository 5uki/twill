# 实现 M3 高价值动作自动标记流
## 目标

在已经具备阅读、手动处理和当前站点筛选的基础上，补齐 M3 的高价值动作闭环：

- 复制验证码后自动标记消息为已处理
- 打开验证链接后自动标记消息为已处理
- 对应 extract 自动从工作台顶部移除
- CLI / Tauri / 前端预览共享同一套动作合同

## 本轮范围

### 交付

- Rust 服务新增消息动作入口
- Tauri 新增 `apply_workspace_message_action`
- CLI 新增 `message action --id <message-id> --action <copy_code|open_link> [--format text|json]`
- 前端详情面板复制验证码 / 打开链接后自动回写 snapshot
- extract 卡片执行动作后同步移除对应条目

### 不在本轮

- 原始邮件外部打开动作
- 站点确认流
- 独立 read/unread 状态
- 48 小时验证窗口

## 合同要求

- `copy_code` 仅允许作用于存在 `extracted_code` 的消息
- `open_link` 仅允许作用于存在 `verification_link` 的消息
- 动作执行成功后必须：
  - 将消息状态更新为 `processed`
  - 重建 message groups / mailbox / site summaries
  - 移除对应 extract
- 返回结构需包含动作结果与更新后的 snapshot，便于 CLI 模拟与前端复用

## 完成标准

- `cargo test --manifest-path src-tauri/Cargo.toml`
  - 覆盖动作执行后的自动标记与 extract 移除
  - 覆盖 CLI `message action`
- `bun test`
  - 覆盖本地动作 helper
- `bun run build`
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`
