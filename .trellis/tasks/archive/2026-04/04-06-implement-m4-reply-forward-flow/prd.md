# 实现 M4 回复 / 转发流

## 目标

在 M4 新建邮件发送首切片基础上，补齐剩余最核心的发送能力：

- 从邮件详情一键进入“回复”或“转发”
- 共享同一套 reply/forward 预填逻辑
- 为这条能力提供 CLI 模拟入口，而不只是在 GUI 中可见

## 本轮范围

### 交付

- Rust 新增结构化 compose prepare 合同
- Rust 服务新增 reply / forward 预填逻辑
- CLI 新增 `compose prepare --mode <reply|forward> --source-message <id> [--format text|json]`
- Tauri 新增 compose prepare command
- 前端详情面板新增“回复 / 转发”动作
- Compose 面板显示当前模式与来源邮件信息

### 不在本轮

- 真正的邮件线程模型
- `Reply-To` / `Cc` / `Bcc`
- 多条转发附件
- HTML 正文和富文本引用样式

## 合同要求

- CLI / Tauri / 前端必须复用同一套 reply/forward 预填语义
- `reply` 默认收件人为原邮件 `sender`
- `forward` 默认保留空收件人
- `reply` / `forward` 的主题前缀不得重复叠加
- 生成的正文必须带原邮件引用块，便于用户继续编辑后发送

## 完成标准

- `bun test`
- `bun run build`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- compose prepare --mode reply --source-message msg_github_security --format json`
