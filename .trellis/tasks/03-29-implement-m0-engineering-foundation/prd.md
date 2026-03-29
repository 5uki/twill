# 实现 M0 工程底座

## Goal

把当前 Tauri 模板项目改造成 Twill 的首个可运行工程底座：

- 移除模板 `greet` 示例与默认 UI
- 建立 `React -> Tauri Command -> Rust Service -> CLI` 的最小纵向链路
- 用同一份 Rust 核心逻辑同时支撑桌面端与 CLI 模拟器
- 落地面向 `Recent verification` 工作台的首版页面壳层
- 搭建最小测试入口，作为后续 M1/M2 迭代基础

## Requirements

- 后端目录从单文件入口演进为：
  - `commands/`
  - `services/`
  - `domain/`
  - `infra/`
  - `cli/`
- 后端提供一个共享服务，用于返回工作台启动快照 `WorkspaceBootstrapSnapshot`
- Tauri Command 与 CLI 必须复用同一个服务层，不允许复制逻辑
- CLI 至少提供一个可验证入口，用于输出当前工作台启动快照
- 前端建立最小应用壳层，替换模板页，默认进入 `Recent verification`
- 前端必须通过调用封装访问 Tauri Command，不直接把 `invoke` 散落在页面组件中
- 页面内容先使用后端返回的静态样例数据驱动，不提前实现真实 IMAP/同步逻辑
- 建立最小测试入口：
  - Rust 侧覆盖服务层和 CLI 层
  - 前端侧提供可运行测试入口，用于验证最小视图模型转换或静态数据适配

## Acceptance Criteria

- [ ] 模板 `greet` command、模板表单和模板样式已移除
- [ ] `src-tauri/src/` 已按职责拆出基础模块，`lib.rs` 不再承载全部逻辑
- [ ] 存在明确的共享类型 `WorkspaceBootstrapSnapshot`
- [ ] Tauri Command 可返回工作台启动快照
- [ ] CLI 可输出同一份工作台启动快照
- [ ] 前端可渲染包含以下区域的工作台壳层：
  - 顶栏
  - 左侧导航
  - 中间列表
  - 右侧详情
- [ ] 首页默认视图为 `Recent verification`
- [ ] 前端调用链路为：页面 -> 调用封装 -> Tauri Command
- [ ] 至少存在一条 Rust 服务测试与一条 CLI 测试
- [ ] 至少存在一条前端测试可被统一脚本执行
- [ ] `cargo test`
- [ ] `cargo fmt --check`
- [ ] `cargo clippy --all-targets --all-features`
- [ ] `bun test`
- [ ] `bun run build`

## Contract Draft

### 数据流

```text
React App Shell
  -> frontend command wrapper
  -> Tauri command: load_workspace_bootstrap
  -> Workspace service
  -> static bootstrap source

CLI: workspace bootstrap
  -> Workspace service
  -> static bootstrap source
```

### 输出契约

`WorkspaceBootstrapSnapshot` 至少包含：

- `app_name`
- `default_view`
- `generated_at`
- `navigation`
- `message_groups`
- `selected_message`

其中：

- `default_view` 在 M0 固定为 `recent_verification`
- `navigation` 用于左侧工作台导航
- `message_groups` 用于中间列表分组
- `selected_message` 用于右侧详情区默认展示

### CLI 契约

- 命令：`workspace bootstrap`
- 可选参数：`--format text|json`
- 默认格式：`text`

## Validation / Error Matrix

### Good

- `workspace bootstrap`
  - 返回文本格式的工作台启动快照摘要
- `workspace bootstrap --format json`
  - 返回可解析的 JSON 快照
- 前端加载启动快照成功
  - 正常渲染默认工作台壳层

### Base

- 启动快照中的列表数据为空
  - 前端仍能渲染空状态壳层，不崩溃
- 详情区缺少验证码但存在验证链接
  - 前端仍能展示动作区和原始摘要

### Bad

- CLI 传入不支持的 `--format`
  - 返回明确错误，而不是静默回退
- Tauri Command 调用失败
  - 前端展示明确错误态

## Technical Notes

- M0 不实现真实账户、同步、提取、分类与缓存
- M0 的样例数据必须放在 Rust 侧，保证 CLI 和桌面端共享同一逻辑来源
- 前端本轮重点是壳层结构、类型边界与调用封装，不做视觉精修
- 如果为测试或视图模型转换创建前端纯函数，优先放在 `src/features/workspace/` 内部，避免过早抽成全局工具
