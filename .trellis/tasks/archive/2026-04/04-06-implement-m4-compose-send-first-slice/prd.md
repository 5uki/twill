# 实现 M4 新建邮件发送首切片

## 目标

在 M3 已完成的前提下，启动 M4 的第一刀：

- 提供“新建邮件”最小可用面板
- 打通统一的发送服务层、CLI 模拟器与 Tauri command
- 在桌面端给出明确的发送结果反馈
- 覆盖关键失败路径：账户不存在、凭据缺失、收件人不合法、SMTP 提交通道不可达

## 本轮范围

### 交付

- Rust 新增结构化发送输入 / 输出模型
- Rust 新增统一发送服务，复用账号元数据与系统安全存储
- CLI 新增 `message send --account <id> --to <email> --subject <subject> --body <text> [--format text|json]`
- Tauri 新增发送 command
- 前端新增独立 Compose 面板
- 前端允许选择账号、输入收件人、主题、正文，并显示发送结果反馈

### 不在本轮

- 回复 / 转发
- 多收件人、抄送、密送
- 富文本编辑器
- 附件
- 完整真实 SMTP 协议适配
- 已发送箱 / 草稿箱持久化

## 合同要求

- CLI / Tauri / 前端必须复用同一套发送服务
- 发送入口必须要求明确的 `account_id / to / subject / body`
- 前端浏览器预览模式不伪造发送成功，应明确提示仅桌面端可发送
- 发送结果必须包含结构化状态、摘要、目标账号与 SMTP 端点信息
- 关键失败路径必须返回结构化错误，而不是模糊字符串

## 完成标准

- `bun test`
- `bun run build`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- message send --account acct_primary-example-com --to dev@example.com --subject "hello" --body "world" --format json`
