# 实现 M3 消息处理流

## 目标

在 M3 阅读首切片已经具备搜索、分类筛选、列表/详情阅读能力的前提下，补齐“消息已处理”这条最短闭环：

- 用户可以在详情面板手动标记某封消息为“已处理”
- 用户也可以把已处理消息撤销回“待处理”
- CLI、Tauri Command、前端桌面端与浏览器预览模式共用同一套状态语义
- 状态变化后，工作台相关派生数据必须一起更新，而不是只改一处文案

## 本轮范围

### 交付

- Rust 服务层新增消息状态更新入口
- Tauri 新增 `update_workspace_message_status`
- CLI 新增：
  - `message mark --id <message-id> --status <pending|processed> [--format text|json]`
- 状态回写后同步重建：
  - `message_groups`
  - `message_details`
  - `selected_message`
  - `mailboxes.unread_count`
  - `site_summaries.pending_count`
- 前端详情面板新增：
  - `标记已处理`
  - `撤销已处理`
- 浏览器预览模式新增本地 snapshot 回写 helper，保证无需桌面 runtime 也能演示同一条流程

### 不在本轮

- 复制验证码 / 打开验证链接后的自动标记
- extract 卡片随消息状态自动消失
- 原始邮件打开动作
- 站点确认流
- 更细的“当前站点”联动

## 合同要求

- `MessageStatus` 当前只允许：
  - `pending`
  - `processed`
- `message mark` 与 `update_workspace_message_status` 必须是幂等的
  - 对已经是目标状态的消息重复执行，不报错
- 状态回写后，消息必须进入对应分组
  - `pending` 留在主分组
  - `processed` 进入次级分组
- 同一个 `message_id` 的 item/detail/selected 三处状态必须保持一致
- mailbox 未读数按 `status == pending` 重新计算
- 站点待处理数按 `site_hint + pending` 重新计算

## 完成标准

- `cargo test --manifest-path src-tauri/Cargo.toml`
  - 覆盖 Rust 服务层状态回写
  - 覆盖 CLI `message mark`
- `bun test`
  - 覆盖前端本地 snapshot 状态回写 helper
- `bun run build`
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`
- CLI 实跑：
  - `message mark --id msg_github_security --status processed --format json`

## 结果摘要

本轮完成后，M3 不再只是“可读”，而是拥有第一条真正的处理流。用户已经可以在工作台里把一封验证邮件从待处理推进到已处理，并看到列表顺序、详情状态、站点待处理数量和 mailbox 未读数一起变化。
