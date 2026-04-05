# 实现 M2 同步缓存底座

## Goal

在 M1 已完成账户接入、安全存储和实时可达性探测的基础上，完成 M2 当前阶段的可交付范围：

- 为工作台建立本地同步缓存
- 提供 `sync run` CLI 与 `sync_workspace` Tauri Command
- 让桌面端工作台优先读取缓存快照，而不是始终直连共享 JSON
- 让缓存快照具备可查询能力，支持 `mailbox list`、`message list`、`message read`

本轮重点是打通 **同步命令 -> 本地缓存 -> 工作台读路径 -> CLI 查询入口** 这条共用主链路。真实 IMAP 收件箱拉取仍留待下一轮。

## Decision

- 当前同步源继续采用 seeded sync source：
  - 复用 `src/data/workspace-bootstrap.json` 作为唯一共享种子
  - 根据已保存账户重新分配 `account_id`、`mailbox_id`、同步状态和导航 badge
  - 生成并持久化本地工作台缓存
- `workspace bootstrap` / `load_workspace_bootstrap` 的职责切换为：
  - 优先读本地缓存
  - 缓存缺失或缓存目录不可访问时退回共享种子
- 桌面端在加载、保存账户成功后和手动刷新时都会复用同一条 `sync_workspace` 逻辑
- 前端顶部栏显示用户可见的同步摘要，并提供“立即同步”入口
- 缓存快照除工作台展示字段外，还保留 `mailboxes`、`message_details` 和更完整的 `sync_status`，供 CLI / 后续能力查询

## Requirements

- 新增工作台缓存仓库，默认路径落到平台持久化目录
- 缓存写入必须具备文件锁和原子替换，不落系统临时目录
- `sync run`、`sync_workspace`、`workspace bootstrap`、`load_workspace_bootstrap` 必须复用同一套服务层逻辑
- 没有已保存账户时，`sync run` / `sync_workspace` 必须明确报错
- `mailbox list`、`message list`、`message read` 必须复用同一份缓存快照，而不是单独维护 mock
- `WorkspaceBootstrapSnapshot`、Rust struct、TypeScript 类型和共享 JSON 必须保持同一份合同
- 浏览器预览继续允许退回到打包种子快照，桌面端优先走缓存

## Acceptance Criteria

- [x] `sync run` CLI 可执行并返回结构化同步快照
- [x] `sync_workspace` Tauri Command 可执行并返回结构化同步快照
- [x] `workspace bootstrap` 可优先读取已写入缓存
- [x] 没有账户时同步会返回明确校验错误
- [x] 顶部栏会显示同步状态，并允许手动触发同步
- [x] `mailbox list` 可读取共享种子或已同步缓存中的邮箱汇总
- [x] `message list` 支持按 `account`、`mailbox`、`verification-only` 查询缓存消息
- [x] `message read` 可返回缓存中的消息详情与正文预抓取状态
- [x] 共享种子 JSON 已补齐 `mailboxes`、`message_details`、`sync_status` 等字段
- [x] Rust 服务层、CLI、缓存仓库有自动化测试覆盖
- [x] `cargo test`
- [x] `cargo fmt --check`
- [x] `cargo clippy --all-targets --all-features`
- [x] `bun test`
- [x] `bun run build`

## Out Of Scope

- 真实 IMAP 协议登录
- 真正的邮件元数据抓取与正文预抓取
- 轮询调度与后台自动同步
- 本地消息索引查询
- 邮件分类、验证码提取和站点沉淀增强

## Notes

- 这轮仍然是 M2 的 seeded cache 阶段，不把 seeded snapshot 冒充为真实邮箱同步结果。
- 当前真实交付的是同步缓存、桌面端读路径、同步状态反馈和缓存查询入口。
