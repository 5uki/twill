# 实现 M3 当前站点匹配流
## 目标

在已经具备阅读与手动处理闭环的前提下，补齐 M3 里“当前站点”这条最关键的上下文链路：

- 顶栏提供独立的 `当前站点` 输入，不与搜索框混用
- 输入域名后优先进行精确匹配
- 未精确命中时展示候选站点，帮助用户快速回填
- CLI / Tauri / 前端浏览器预览共享同一套站点解析与筛选规则

## 本轮范围

### 交付

- Rust 服务新增站点上下文解析入口
- Tauri 新增 `resolve_workspace_site_context`
- CLI 新增：
  - `site-context resolve --domain <domain> [--format text|json]`
  - `message list --site <hostname>` 精确站点过滤
- 前端顶栏新增：
  - `当前站点` 输入
  - 站点命中提示
  - 候选站点按钮
- 前端阅读流在精确命中时按站点过滤消息

### 不在本轮

- “加入站点清单”确认流
- 原始邮件打开动作
- 自动标记已处理
- 已读/未读独立状态
- 48 小时最近验证窗口约束

## 合同要求

- 输入支持手动域名与 URL 粘贴，解析后统一归一化为 hostname
- 归一化至少处理：
  - 协议头
  - 路径 / query / hash
  - `www.`
  - 端口
- `message list --site` 只做精确站点过滤，不做模糊匹配
- `site-context resolve`：
  - 精确命中时返回 `matched_site`
  - 未命中时返回按相关性排序的 `candidate_sites`
- 浏览器预览模式必须有本地 helper，可在没有桌面 runtime 时演示同样逻辑

## 完成标准

- `cargo test --manifest-path src-tauri/Cargo.toml`
  - 覆盖 Rust 站点解析与站点消息过滤
  - 覆盖 CLI `site-context resolve`
- `bun test`
  - 覆盖本地 `current site` 解析与筛选 helper
- `bun run build`
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`
