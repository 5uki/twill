# 实现 M3 站点确认流

## 目标

在已具备当前站点输入、精确匹配和候选站点提示的基础上，补齐 M3 的“加入站点清单”确认流：

- 手动输入的新域名可以显式加入站点清单
- CLI / Tauri / 前端头部共用同一套确认合同
- 确认后当前站点输入会回到归一化域名，并能立即命中

## 本轮范围

### 交付

- Rust 服务复用并开放 `confirm_workspace_site`
- Tauri 新增 `confirm_workspace_site`
- CLI 新增 `site-context confirm --domain <domain> [--label <label>] [--format text|json]`
- 前端 `TopHeader` 在未命中且输入像完整域名时显示“加入网站清单”
- `App.tsx` 接入确认动作后的 snapshot 回写

### 不在本轮

- 站点标签重命名编辑
- 自动站点识别沉淀
- 48 小时验证窗口
- 原始邮件打开动作

## 合同要求

- 仅当输入可归一化为完整域名时允许确认加入
- 已存在站点再次确认时，允许更新标签但不重复追加
- 确认后必须持久化到 workspace snapshot 的 `site_summaries`
- 浏览器预览模式也必须基于当前内存 snapshot 回写，不能退回初始 seed

## 完成标准

- `cargo test --manifest-path src-tauri/Cargo.toml site_context_confirm_adds_manual_site_to_snapshot`
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- site-context confirm --domain https://vercel.com/login --format json`
- `bun run build`
