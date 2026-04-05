# 实现 M3 独立已读状态流

## 目标

在已有阅读、处理、站点上下文和高价值动作闭环的基础上，补齐 M3 中独立的 `read/unread` 能力：

- 消息可以单独标记为已读或未读
- `read_state` 与 `pending/processed` 解耦
- CLI / Tauri / 前端详情面板共用同一套合同

## 本轮范围

### 交付

- Rust 服务新增 `update_workspace_message_read_state`
- Tauri 新增 `update_workspace_message_read_state`
- CLI 新增 `message read-state --id <message-id> --state <unread|read> [--format text|json]`
- 前端详情面板新增“标记已读 / 标记未读”
- 浏览器预览模式复用本地 `applyWorkspaceMessageReadState`

### 不在本轮

- 键盘快捷键
- 窄窗口 Drawer 退化
- 提取失败后的特殊回退 UI

## 合同要求

- `read_state` 只修改已读状态，不隐式改动 `status`
- 更新后必须同步刷新列表项、详情项、选中消息与邮箱未读数
- `message open` / `message original` 仍自动标记为 `read`
- `message mark processed` / `message action` 仍自动将消息置为 `read`

## 完成标准

- `bun test`
- `bun run build`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- message read-state --id msg_github_security --state read --format json`
