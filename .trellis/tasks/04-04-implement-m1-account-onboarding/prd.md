# 实现 M1 账户接入

## Goal

启动 Twill 的 M1 里程碑，并在进入真实账户接入前补齐当前工作台 UI 的动画基线。

本任务的目标不是一次性做完整个 M1，而是确定并落地一个可执行、可测试、可继续迭代的第一切片，让账户接入从现有 M0 静态工作台自然演进，而不是另起一套界面或协议。

## What I already know

- 用户希望“开始 M1”。
- 用户明确提出：已完成的 M0 需要回补 UI 动画效果，而且这也是重点。
- 规划文档中，M1 当前定义包含：
  - 账户模型
  - 通用 `IMAP/SMTP + 手动配置` 接入
  - 连接测试命令与 CLI
  - 本地安全存储策略
  - 多账号元数据与隔离边界
- 当前仓库已经完成 M0 的基础工程链路：
  - React 页面壳层
  - 前端调用封装
  - Tauri command `load_workspace_bootstrap`
  - Rust service
  - CLI `workspace bootstrap`
- 当前前端工作台 UI 已有结构、视觉和信息层级，但没有正式记录动画规范，也没有明显的动态过渡实现。
- 当前 UI 规划已确认：
  - 首页为 `Recent verification`
  - 三栏工作台结构
  - 高密度列表
  - 右侧详情与快捷动作固定
- 后端硬约束已明确：
  - 严格 TDD
  - 所有功能必须提供 CLI 模拟器
  - CLI / Tauri Command / 服务层必须复用同一套核心逻辑
- 当前任务是明显的跨层任务：
  - 前端 UI 与交互
  - Tauri command 契约
  - Rust 领域模型 / 服务
  - CLI 模拟入口

## Assumptions (temporary)

- 本轮更合理的做法是先确定并实现 M1 的第一切片，而不是一次吞下整个 M1。
- M0 回补的动画将作为后续 M1 界面继续沿用的工作台动效基线，而不是一次性的装饰效果。
- 动画应以“帮助扫读、切换和状态反馈”为目标，避免为了炫技引入重型视觉干扰。
- 动画设计必须考虑 `prefers-reduced-motion` 或等价的降级策略。
- 安全存储的最终实现方式仍需在确定 M1 第一切片时一起收敛。

## Decision (ADR-lite)

**Context**

- 用户已选择 M1 的“接入首切片”。
- 当前 M0 已有工作台壳层与 `workspace bootstrap` 链路，但还没有账户接入能力。
- 安全存储尚未进入本轮范围，因此不能为了赶进度把敏感信息以临时不安全方式落地。

**Decision**

- 本轮实现 M1 第一切片为：
  - 账户模型
  - 账户面板 / 表单 UI
  - `account add`
  - `account list`
  - `account test`
  - 对应的 Tauri command / 服务层 / CLI 模拟器
  - M0 工作台 UI 动画补齐
- 本轮 `account test` 的语义定义为：**手动配置连接预检**。
  - 重点验证输入契约、协议配置组合、端口 / 安全策略匹配和跨层结果展示
  - 不在本轮引入真正的密钥持久化与完整 IMAP/SMTP 网络握手

**Consequences**

- 我们可以尽快建立稳定的账户接入契约、服务边界和 UI 入口。
- CLI / Tauri / 前端会先对齐一套真正可跑的账户链路。
- 后续进入安全存储与真实协议接入时，可以在不推翻当前契约主干的前提下增强实现。
- 本轮需要在文档和界面上明确“连接预检”语义，避免误导为完整联网登录验证。

## Requirements (evolving)

- 需要为 M1 建立独立任务与 PRD，并明确它与 M0 的承接关系。
- M1 的第一切片必须是可运行、可测试、可通过 CLI 观察结果的，不接受只做界面草图或只做局部类型定义。
- M1 第一切片固定为：
  - 账户模型
  - 账户面板与表单
  - `account add/list/test`
  - Tauri command / 服务层 / CLI
  - M0 UI 动画补齐
- 新增或修改的跨层契约必须明确：
  - 输入字段
  - 输出字段
  - 校验位置
  - 错误模型
- 前端必须在现有工作台基础上演进，不另起完全独立的新壳层。
- M0 回补动画必须纳入当前任务范围，并作为重点交付项之一。
- 动画至少应覆盖以下高频交互或状态：
  - 页面进入
  - 顶栏 / 导航 / 列表 / 详情的分层入场
  - 列表项 hover / select / processed 状态切换
  - 右侧详情与动作卡片的显隐或内容切换
  - loading / error / ready 状态切换
- 动画应服务于工作台效率：
  - 快
  - 克制
  - 有层级
  - 不拖慢操作
- 如果 M1 第一切片涉及账户操作，则必须同步提供：
  - Tauri command
  - CLI 命令
  - 服务层复用
  - 自动化测试
- 由于安全存储不在本轮范围：
  - 账户新增只保存非敏感元数据与服务器配置
  - `account test` 不依赖已持久化密码
  - 测试结果必须明确表现为“预检”而非“已完成真实登录”

## Acceptance Criteria (evolving)

- [ ] 已明确 M1 第一切片的范围边界
- [ ] 已明确需要新增或变更的跨层契约
- [ ] 已明确动画补齐的目标区域与最低交付标准
- [ ] M1 第一切片可通过 CLI 执行 `account add/list/test` 关键路径
- [ ] Tauri command 与 CLI 复用同一套服务逻辑
- [ ] 至少存在一条失败测试先于或伴随实现补齐
- [ ] 前端工作台具有成体系的进入 / 切换 / 状态反馈动画
- [ ] 动画存在降级方案，不强制所有用户承受完整动效
- [ ] 前端 `Accounts` 视图可执行账户新增与连接预检
- [ ] 账户列表可反映已保存的非敏感账户元数据
- [ ] `account test` 的结果会清晰区分通过 / 失败及原因

## Definition of Done

- 测试已补齐并可运行
- CLI / Tauri / 前端调用链已对齐
- 新增 UI 动画已和现有工作台结构融合，而不是零散特效
- 关键实现与范围边界已回写到任务文档
- lint / typecheck / 相关测试通过

## Out of Scope (initial)

- 一次性完成 M1 全部能力
- 在未收敛切片前直接接入完整 IMAP 同步链路
- 本轮接入系统级安全存储
- 本轮接入完整 IMAP/SMTP 联网握手
- 为兼容旧协议或旧 UI 额外保留双轨逻辑
- 纯品牌化视觉翻新
- 与当前工作台无关的大面积 UI 重构

## Technical Notes

### 当前已知代码入口

- 前端：
  - `src/App.tsx`
  - `src/App.css`
  - `src/features/workspace/WorkspaceShell.tsx`
  - `src/features/workspace/model.ts`
  - `src/features/workspace/view-model.ts`
  - `src/lib/tauri/loadWorkspaceBootstrap.ts`
- 后端：
  - `src-tauri/src/commands/workspace.rs`
  - `src-tauri/src/services/workspace_service.rs`
  - `src-tauri/src/domain/workspace.rs`
  - `src-tauri/src/domain/error.rs`
  - `src-tauri/src/infra/static_workspace.rs`
  - `src-tauri/src/cli/mod.rs`

### 当前已知模式

- 当前 Tauri command 仅承载薄装配，服务层返回领域结构，符合继续扩展的方向。
- CLI 已具备基础参数解析与格式化输出，适合沿用到 M1。
- 前端当前仍主要由静态快照驱动，账户接入与真实交互尚未开始。
- 当前工作台样式集中在 `src/App.css`，动画补齐大概率也会先从该文件和相关组件状态类开始。

## Technical Approach

### Slice A：账户跨层契约与服务

- 新增账户领域模型、输入结构、输出结构与错误语义
- 新增账户服务层
- 新增非敏感账户元数据存储
- 新增手动配置连接预检逻辑

### Slice B：Tauri command 与 CLI

- 新增 `add_account`
- 新增 `list_accounts`
- 新增 `test_account_connection`
- CLI 新增：
  - `account add`
  - `account list`
  - `account test`

### Slice C：Accounts 视图

- 在现有工作台 `Accounts` 导航下接入账户面板
- 提供：
  - 基础表单
  - 账户列表
  - 连接预检结果展示
- 不创建全新应用壳层，继续复用现有工作台框架

### Slice D：M0 动画补齐

- 为顶栏、导航、中心面板、详情面板建立分层入场动画
- 为卡片 hover / active / processed 状态建立过渡
- 为 loading / error / ready 建立统一动效节奏
- 提供 `prefers-reduced-motion` 降级

### 当前约束

- `src-tauri/Cargo.toml` 存在用户未提交修改，处理 M1 时需要避免覆盖。
- 前端规范文档目前仍是模板占位，实际实现需要以现有代码模式和已确认 UI 文档为准。
- M1 涉及跨层契约与安全存储时，需要先做契约深度检查，再决定是否进入真实协议接入。
