# Account & Workspace Contracts

> 适用范围：[src-tauri/src/commands/account.rs](../../../src-tauri/src/commands/account.rs)、[src-tauri/src/commands/workspace.rs](../../../src-tauri/src/commands/workspace.rs)、[src-tauri/src/commands/sync.rs](../../../src-tauri/src/commands/sync.rs)、[src-tauri/src/services/account_service.rs](../../../src-tauri/src/services/account_service.rs)、[src-tauri/src/services/workspace_service.rs](../../../src-tauri/src/services/workspace_service.rs)、[src-tauri/src/cli/mod.rs](../../../src-tauri/src/cli/mod.rs)、[src/lib/app-api.ts](../../../src/lib/app-api.ts)、[src/components/account-form.ts](../../../src/components/account-form.ts)

---

## 场景一：账户接入、凭据存储与实时探测

### 1. Scope / Trigger

- 触发条件：修改 `list_accounts`、`add_account`、`test_account_connection` 任一 Tauri Command / CLI 命令 / 服务层函数。
- 触发条件：修改账户元数据 JSON、系统安全存储、账户 ID 生成、前端表单到命令入参映射。

### 2. Signatures

Rust Tauri Command：
```rust
#[tauri::command]
pub fn list_accounts<R: Runtime>(app: AppHandle<R>) -> Result<Vec<AccountSummary>, AppError>

#[tauri::command]
pub fn add_account<R: Runtime>(
    app: AppHandle<R>,
    input: AddAccountInput,
) -> Result<AccountSummary, AppError>

#[tauri::command]
pub async fn test_account_connection(
    input: AccountConnectionTestInput,
) -> Result<AccountConnectionTestResult, AppError>
```

CLI：
```text
account list [--format text|json]
account add --name <name> --email <email> --login <login> --password <password> \
  --imap-host <host> --imap-port <port> --imap-security <none|start_tls|tls> \
  --smtp-host <host> --smtp-port <port> --smtp-security <none|start_tls|tls> \
  [--format text|json]
account test --name <name> --email <email> --login <login> \
  --imap-host <host> --imap-port <port> --imap-security <none|start_tls|tls> \
  --smtp-host <host> --smtp-port <port> --smtp-security <none|start_tls|tls> \
  [--format text|json]
```

前端封装：
```ts
listAccounts(): Promise<AccountSummary[]>
addAccount(input: AddAccountCommandInput): Promise<AccountSummary>
testAccountConnection(
  input: AccountConnectionCommandInput,
): Promise<AccountConnectionTestResult>
```

### 3. Contracts

新增账户请求 `AddAccountInput`：
```json
{
  "display_name": "Primary Gmail",
  "email": "primary@example.com",
  "login": "primary@example.com",
  "password": "app-password",
  "imap": { "host": "imap.example.com", "port": 993, "security": "tls" },
  "smtp": { "host": "smtp.example.com", "port": 587, "security": "start_tls" }
}
```

连接测试请求 `AccountConnectionTestInput`：
```json
{
  "display_name": "Primary Gmail",
  "email": "primary@example.com",
  "login": "primary@example.com",
  "imap": { "host": "imap.example.com", "port": 993, "security": "tls" },
  "smtp": { "host": "smtp.example.com", "port": 587, "security": "start_tls" }
}
```

关键合同：
- `AccountConnectionTestInput` 没有 `password` 字段；当前测试只覆盖身份格式校验和 socket 可达性，不代表真实 IMAP / SMTP 认证成功。
- 前端表单允许 `login` 留空，但 [account-form.ts](../../../src/components/account-form.ts) 必须在调用前自动回退为 `email`；Rust 命令契约仍要求收到非空 `login`。
- `AccountSummary.id` 由 `email` 归一化生成，格式为 `acct_<normalized-email>`，不能退回到基于序号的 ID。
- 账户元数据落盘路径优先级：`TWILL_ACCOUNT_STORE` 环境变量 > 平台应用数据目录下的 `Twill/accounts.json`。
- `accounts.json` 只保存元数据，绝不能写入明文密码；密码只进入系统安全存储。
- `list_accounts` 返回前必须根据系统安全存储结果重算 `credential_state`，不能盲信 JSON 原始值。
- `add_account` 的顺序固定为：校验输入 -> 写元数据 -> 写系统密码；若密码写入失败则回滚元数据。
- `JsonFileAccountRepository` 必须通过文件锁串行化读写，并通过临时文件替换实现原子写。

### 4. Validation & Error Matrix

| 入口 | 条件 | 结果 |
|------|------|------|
| `add_account` / `account add` | `display_name` 为空 | `Validation(field="display_name")` |
| `add_account` / `account add` | `email` 为空或不含 `@` | `Validation(field="email")` |
| `add_account` / `account add` | `login` 为空 | `Validation(field="login")` |
| `add_account` / `account add` | `password` 为空 | `Validation(field="password")` |
| `add_account` / `account add` | `imap.host` / `smtp.host` 为空 | `Validation(field="<prefix>.host")` |
| `add_account` / `account add` | host 含空白或不含 `.` | `Validation(field="<prefix>.host")` |
| `add_account` / `account add` | 端口为 `0` | `Validation(field="<prefix>.port")` |
| `add_account` / `account add` | 邮箱已存在 | `Validation(field="email")` |
| `add_account` | 元数据写入失败 | `Storage`，且不得写入系统密码 |
| `add_account` | 密码写入失败 | 返回原错误，且必须删除刚写入的元数据 |
| `list_accounts` / `add_account` | 文件锁等待超过 5 秒 | `Storage` |
| `test_account_connection` / `account test` | 输入校验失败 | 同上游 `Validation` |
| `test_account_connection` | 主机可解析但端口不可达 | `AccountConnectionTestResult.status = "failed"` |
| `test_account_connection` | 主机和端口可达 | `AccountConnectionTestResult.status = "passed"` |

### 5. Good / Base / Bad Cases

#### Good

- 前端留空 `login` 时，`buildAccountCommandInput()` 自动补成邮箱地址，再调用 `add_account`。
- `add_account` 成功后，`list_accounts` 返回的 `credential_state` 为 `stored`。
- `account test --format json` 返回 `status / summary / checks[]`，其中 `checks.target` 只允许 `identity | imap | smtp`。

#### Base

- 浏览器预览模式下可以演示账户管理界面，但不应伪造系统密码保存能力。
- `test_account_connection` 只验证主机、端口和安全策略组合；它是“可达性探测”，不是“账户登录”。

#### Bad

- 让前端直接发送空 `login` 到 Rust 命令。
- 在 JSON 元数据中保存明文密码。
- 根据当前账户数量生成 `acct_1`、`acct_2` 这类易冲突 ID。
- 把 `test_account_connection` 当作真实邮箱登录成功的证明。

### 6. Tests Required

- `cargo test --manifest-path src-tauri/Cargo.toml`
  - 服务层：新增账户、重复邮箱、元数据失败、密码失败回滚、连接探测成功 / 失败。
  - 基础设施层：文件锁、并发保存、原子写。
  - 系统安全存储集成测试在 Linux runner 缺少 `org.freedesktop.secrets` 后端时可以显式跳过，但不能把真实业务路径静默改成“假成功”。
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- account add ...`
  - 断言 CLI 与服务层共用同一合同。
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- account test ... --format json`
  - 断言 JSON 结构和成功 / 失败语义。
- `bun test`
  - 断言 [account-form.ts](../../../src/components/account-form.ts) 的 `login` 自动回退逻辑与 Rust 合同一致。

### 6.5. M3 阅读首切片补充合同（2026-04-05）

本轮在 M2 已有的缓存查询基础上，为阅读体验新增以下 **必须同步** 的合同：

- Rust Tauri Command：
  - `list_workspace_messages(filter?: WorkspaceMessageFilter) -> Result<Vec<WorkspaceMessageItem>, AppError>`
  - `read_workspace_message(message_id: String) -> Result<WorkspaceMessageDetail, AppError>`
- CLI：
  - `message list --category <registration|security|marketing> --query <keyword> [--format text|json]`
  - 仍与 `--account`、`--mailbox`、`--verification-only` 组合使用
- 前端：
  - `listWorkspaceMessages(filter?: WorkspaceMessageFilter): Promise<WorkspaceMessageItem[]>`
  - `readWorkspaceMessage(messageId: string): Promise<WorkspaceMessageDetail>`
  - 桌面端优先走 Tauri command；浏览器预览回退到共享快照，但字段名和过滤语义必须保持一致

`WorkspaceMessageFilter` 当前字段：

```json
{
  "account_id": "optional",
  "mailbox_kind": "inbox | spam_junk | null",
  "verification_only": true,
  "category": "registration | security | marketing | null",
  "query": "keyword"
}
```

过滤语义：

- `verification_only=true` 只保留 `has_code || has_link` 的消息
- `category` 精确匹配 `WorkspaceMessageItem.category`
- `query` 以 **大小写不敏感** 方式匹配：
  - `subject`
  - `sender`
  - `preview`
  - `account_name`
  - `mailbox_label`
- `read_workspace_message` / `message read` 的 `message_id` 合同继续保持不变，桌面端详情面板与 CLI 必须读取同一份 `WorkspaceMessageDetail`

新增验证矩阵：

| 入口 | 条件 | 结果 |
|------|------|------|
| CLI `message list` | `--category` 非 `registration|security|marketing` | `Validation(field="category")` |
| 桌面端 `listWorkspaceMessages()` | Tauri invoke 失败 | 前端允许回退到当前快照过滤结果，但必须显示用户可理解的错误提示 |

新增最小测试要求：

- `cargo test --manifest-path src-tauri/Cargo.toml`
  - 覆盖 `filters_messages_by_category_and_query`
  - 覆盖 CLI `message_list_supports_category_and_query_filters`
- `bun test`
  - 覆盖前端本地阅读辅助逻辑对 `category/query` 的过滤
  - 覆盖“选中消息已被过滤掉时，回退到首条可见消息”的选择语义

### 7. Wrong vs Correct

#### Wrong

- “前端字段留空没关系，Rust 自己猜就行。”
- “测试连接通过了，所以邮箱登录已经打通。”
- “把账户配置写进临时目录，开发期先凑合。”

#### Correct

- 前端在调用前补齐 `login`，Rust 只接收明确合同。
- `test_account_connection` 明确定义为可达性探测，真实收件箱同步要走另一套实现。
- 账户元数据落到持久化目录，密码单独进系统安全存储。

---

## 场景二：工作台快照加载、同步缓存与当前阶段边界

### 1. Scope / Trigger

- 触发条件：修改 `load_workspace_bootstrap`、`sync_workspace`、`workspace bootstrap` CLI、`sync run` CLI、`mailbox list`、`message list`、`message read`、`WorkspaceBootstrapSnapshot` 结构、`src/data/workspace-bootstrap.json`。
- 触发条件：修改前端工作台对 `navigation / mailboxes / message_groups / message_details / extracts / site_summaries / sync_status` 的消费方式。
- 触发条件：修改工作台缓存文件路径、同步源、缓存回退逻辑或缓存查询能力。

### 2. Signatures

Rust：
```rust
#[tauri::command]
pub fn load_workspace_bootstrap() -> Result<WorkspaceBootstrapSnapshot, AppError>

#[tauri::command]
pub async fn sync_workspace() -> Result<WorkspaceBootstrapSnapshot, AppError>

pub fn load_workspace_bootstrap<R>(repository: &R) -> Result<WorkspaceBootstrapSnapshot, AppError>
pub fn sync_workspace<A, R, S>(
  account_repository: &A,
  workspace_repository: &R,
  sync_source: &S,
) -> Result<WorkspaceBootstrapSnapshot, AppError>
pub fn list_workspace_mailboxes<R>(repository: &R) -> Result<Vec<WorkspaceMailboxSummary>, AppError>
pub fn list_workspace_messages<R>(
  repository: &R,
  filter: &WorkspaceMessageFilter,
) -> Result<Vec<WorkspaceMessageItem>, AppError>
pub fn read_workspace_message<R>(
  repository: &R,
  message_id: &str,
) -> Result<WorkspaceMessageDetail, AppError>
```

CLI：
```text
workspace bootstrap [--format text|json]
sync run [--format text|json]
mailbox list [--format text|json]
message list [--account <account-id>] [--mailbox <inbox|spam_junk>] \
  [--verification-only <true|false>] [--format text|json]
message read --id <message-id> [--format text|json]
```

前端：
```ts
loadWorkspaceBootstrap(): Promise<WorkspaceBootstrapSnapshot>
syncWorkspace(): Promise<WorkspaceBootstrapSnapshot>
```

### 3. Contracts

`WorkspaceBootstrapSnapshot` 当前字段：
```json
{
  "app_name": "Twill",
  "generated_at": "2026-04-05T00:00:00Z",
  "default_view": "recent_verification",
  "navigation": [],
  "mailboxes": [],
  "message_groups": [],
  "selected_message": {},
  "message_details": [],
  "extracts": [],
  "site_summaries": [],
  "sync_status": {
    "state": "ready",
    "summary": "已同步 1 个账号，共 3 封邮件",
    "phase": "first",
    "poll_interval_minutes": 3,
    "retention_days": 30,
    "next_poll_at": "2026-04-05T00:03:00Z",
    "folders": ["Inbox", "Spam/Junk"]
  }
}
```

边界说明：
- `workspace bootstrap` / `load_workspace_bootstrap` 现在优先读取本地工作台缓存；缓存缺失时才退回共享种子文件 [workspace-bootstrap.json](../../../src/data/workspace-bootstrap.json)。
- `load_workspace_bootstrap` 遇到缓存目录不可访问这类 `Storage` 错误时允许回退到共享种子；遇到缓存序列化损坏等非存储错误时应继续暴露错误，而不是静默吞掉。
- `sync run` / `sync_workspace` 会读取当前已保存账户，并基于共享种子文件生成一份“已同步缓存快照”，然后写入本地缓存。
- 已同步缓存快照会补齐 `account_id`、`mailbox_id`、`mailbox_label`、`message_details`、`mailboxes` 与更完整的 `sync_status`，供 CLI 和后续功能查询。
- `mailbox list`、`message list`、`message read` 与 `workspace bootstrap` 必须读取同一份缓存快照；缓存缺失时沿用同样的共享种子回退逻辑。
- 已同步缓存快照可以附带 `sync_status`，供桌面端顶部栏显示用户视角的同步状态，而不是暴露 `generated_at` 或底层实现术语。
- 当前这份“已同步缓存快照”仍然是 **seeded sync source**，重点是打通 M2 的缓存、CLI、Tauri 和工作台读路径；它还不是最终的真实 IMAP 拉取结果。
- 共享种子文件必须同时被前端 import、Rust `include_str!` 和 seeded sync source 消费，避免双份 mock / seed 漂移。
- `loadWorkspaceBootstrap()` 在桌面端优先走 Tauri Command；不在 Tauri 环境或命令失败时，前端退回到打包内的同一份 JSON。
- `generated_at` 可以保留在快照结构里供调试 / CLI 使用，但默认用户界面不应把它当作产品信息展示。

### 4. Validation & Error Matrix

| 入口 | 条件 | 结果 |
|------|------|------|
| CLI `sync run` / Tauri `sync_workspace` | 没有已保存账户 | `Validation(field="accounts")` |
| CLI `sync run` / Tauri `sync_workspace` | 工作台缓存写入失败 | `Storage` |
| Rust `load_workspace_bootstrap` | JSON 非法 | 解析 panic，必须在测试 / 构建阶段暴露 |
| CLI `workspace bootstrap --format json` | `--format` 非 `text|json` | `UnsupportedFormat` |
| CLI `sync run --format json` | `--format` 非 `text|json` | `UnsupportedFormat` |
| CLI `mailbox list --format json` | `--format` 非 `text|json` | `UnsupportedFormat` |
| CLI `message list` | `--mailbox` 非 `inbox|spam_junk` | `Validation(field="mailbox")` |
| CLI `message list` | `--verification-only` 非 `true|false` | `Validation(field="--verification-only")` |
| CLI `message read` | 缺少 `--id` | `InvalidCliArgs` |
| CLI `message read` | 消息不存在 | `Validation(field="message.id")` |
| 前端 `loadWorkspaceBootstrap()` | 非桌面环境 | 直接返回打包快照 |
| 前端 `loadWorkspaceBootstrap()` | Tauri invoke 失败 | 返回打包快照，界面仍可加载 |
| 前端 `syncWorkspace()` | invoke 失败 | 由调用方决定回退到 `loadWorkspaceBootstrap()`，不应让桌面壳层白屏 |

### 5. Good / Base / Bad Cases

#### Good

- 修改 `WorkspaceBootstrapSnapshot` 后，同时更新 Rust struct、TypeScript 类型、共享 JSON 和同步缓存读写逻辑。
- 浏览器预览继续读取共享种子，桌面端优先读取同步缓存，两者字段结构保持同一合同。
- `sync run` 成功后，`workspace bootstrap`、`mailbox list`、`message list`、`message read` 都能读取到同一份已持久化缓存。

#### Base

- 账户列表已经是真实数据；工作台读路径已经切到本地同步缓存，但缓存内容当前仍由 seeded source 生成。
- 这意味着 M2 已经开始接管“同步 + 缓存 + 读路径 + 查询入口”，但真实 IMAP 拉取仍是下一条明确边界。

#### Bad

- 在前端单独维护另一份 `mockMessages`。
- 误把 `sync run` 当前的 seeded snapshot 当成真实 IMAP 收件箱同步。
- 改了 Rust struct 但没改 `src/lib/app-types.ts` 或共享 JSON。
- 为 `mailbox list` / `message list` / `message read` 单独复制一套不同的数据源。

### 6. Tests Required

- `cargo test --manifest-path src-tauri/Cargo.toml`
  - 断言默认视图、导航项、消息组能从共享 JSON 成功加载。
  - 断言没有账户时 `sync_workspace` / `sync run` 会拒绝同步。
  - 断言同步后缓存可被 `workspace bootstrap` 重新读取。
  - 断言 `mailbox list`、`message list`、`message read` 能读取静态种子或已同步缓存。
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- sync run --format json`
  - 断言 CLI 输出字段与 JSON 合同一致，并会写入本地缓存。
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- workspace bootstrap --format json`
  - 断言 CLI 会优先读取已写入缓存，而不是直接退回共享种子。
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- mailbox list --format json`
  - 断言邮箱汇总来自同一份工作台快照。
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- message list ... --format json`
  - 断言账号、邮箱与 verification-only 过滤符合合同。
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- message read --id <message-id> --format json`
  - 断言消息详情包含站点提示、正文预抓取标记和同步时间。
- `bun test`
  - 断言工作台分组、收件箱行渲染、同步状态文案和用户文案约束。

### 7. Wrong vs Correct

#### Wrong

- “先在 React 里手写一份 mock，Rust 那边以后再补。”
- “`sync run` 既然能跑通，说明真实 IMAP 拉取已经完成。”

#### Correct

- 共享种子快照只保留一份，前后端与 seeded sync source 共用。
- 在产品说明和任务记录里明确：当前真实的是缓存读写链路、同步命令和缓存查询入口，真实 IMAP 邮件拉取仍未交付。
### 6.6. M3 消息处理流补充合同（2026-04-05）

本轮在 M3 阅读首切片基础上，补齐“手动标记已处理 / 撤销已处理”的最小闭环。以下合同必须保持一致：

- Rust Tauri Command：
  - `update_workspace_message_status(message_id: String, status: MessageStatus) -> Result<WorkspaceBootstrapSnapshot, AppError>`
- CLI：
  - `message mark --id <message-id> --status <pending|processed> [--format text|json]`
- 前端：
  - `updateWorkspaceMessageStatus(messageId: string, status: MessageStatus): Promise<WorkspaceBootstrapSnapshot>`
  - 浏览器预览模式使用本地 helper 回写当前 snapshot，而不是退回固定种子快照

`MessageStatus` 当前只允许：

```json
"pending" | "processed"
```

状态回写语义：

- `message_id` 必须命中现有消息，否则返回 `Validation(field="message.id")`
- 状态更新必须同步作用于：
  - `message_groups[].items[].status`
  - `message_details[].status`
  - `selected_message.status`
- 状态更新后必须重建派生数据：
  - `message_groups`
  - `mailboxes`
  - `site_summaries`
- `mailboxes.unread_count` 以 `status == pending` 为准重新计算
- `site_summaries.pending_count` 以同站点 `pending` 消息数重新计算
- `message mark` / `update_workspace_message_status` 允许幂等调用；如果目标状态未变化，不应报错

新增验证矩阵：

| 入口 | 条件 | 结果 |
|------|------|------|
| CLI `message mark` | 缺少 `--id` | `InvalidCliArgs` |
| CLI `message mark` | 缺少 `--status` | `InvalidCliArgs` |
| CLI `message mark` | `--status` 非 `pending|processed` | `Validation(field="status")` |
| CLI / Tauri | `message_id` 不存在 | `Validation(field="message.id")` |
| 前端浏览器预览 | 标记状态后再次筛选/选中 | 必须基于当前内存 snapshot，而不是重新退回初始 seed |

新增最小测试要求：

- `cargo test --manifest-path src-tauri/Cargo.toml`
  - 覆盖服务层 `updates_message_status_and_rebuilds_workspace_snapshot`
  - 覆盖 CLI `message_mark_updates_snapshot_and_persists_status_change`
- `bun test`
  - 覆盖本地 helper `applyWorkspaceMessageStatus`
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- message mark --id msg_github_security --status processed --format json`
  - 断言返回快照中的：
    - `selected_message.status == "processed"`
    - `site_summaries[github].pending_count == 0`

### 6.7. M3 当前站点匹配流补充合同（2026-04-05）
本轮补齐 M3 里“当前站点”这条最小上下文链路，要求 CLI、Tauri 与前端本地预览共享同一套域名解析和站点筛选语义。

- Rust / Tauri：
  - `resolve_workspace_site_context(input: String) -> Result<WorkspaceSiteContextResolution, AppError>`
- CLI：
  - `site-context resolve --domain <domain> [--format text|json]`
  - `message list --site <hostname> [--format text|json]`
- 前端：
  - `resolveWorkspaceSiteContext(input: string): Promise<WorkspaceSiteContextResolution>`
  - 本地 helper `resolveWorkspaceSiteContext(snapshot, input)`
  - `WorkspaceMessageFilter.site_hint?: string | null`

`WorkspaceSiteContextResolution` 当前合同：

```json
{
  "input": "https://www.github.com/login",
  "normalized_domain": "github.com",
  "matched_site": {
    "id": "site_github",
    "label": "GitHub",
    "hostname": "github.com",
    "pending_count": 1,
    "latest_sender": "noreply@github.com"
  },
  "candidate_sites": []
}
```

解析与过滤语义：

- 输入允许直接粘贴 URL，归一化后必须至少去掉：
  - 协议头
  - 路径 / query / hash
  - `www.`
  - 端口
- `message list --site` / `WorkspaceMessageFilter.site_hint` 只做**精确站点过滤**
- 站点过滤不依赖 `WorkspaceMessageItem` 额外字段，而是基于 `message_details[].site_hint` 与 `selected_message.site_hint` 建立消息到站点的映射
- 精确命中时返回 `matched_site`
- 未命中时返回排序后的 `candidate_sites`
- 浏览器预览模式必须使用本地 helper，不能退回写死的另一套站点 mock 逻辑

新增最小验证要求：

- `cargo test --manifest-path src-tauri/Cargo.toml`
  - 覆盖 `filters_messages_by_exact_site_hint`
  - 覆盖 `resolves_current_site_context_with_exact_match_and_candidates`
  - 覆盖 CLI `message_list_supports_exact_site_filter`
  - 覆盖 CLI `site_context_resolve_returns_exact_match_and_candidates`
- `bun test`
  - 覆盖本地 `resolveWorkspaceSiteContext`
  - 覆盖本地 `site_hint` 精确过滤
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- site-context resolve --domain https://www.github.com/login --format json`
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- message list --site github.com --format json`

### 6.8. M3 高价值动作自动标记补充合同（2026-04-05）

本轮补齐 M3 里“复制验证码 / 打开验证链接后自动标记已处理”的闭环，要求 Rust 服务、Tauri command、CLI 与前端本地预览共享同一套动作语义。

- Rust Tauri Command：
  - `apply_workspace_message_action(message_id: String, action: WorkspaceMessageAction) -> Result<WorkspaceMessageActionResult, AppError>`
- CLI：
  - `message action --id <message-id> --action <copy_code|open_link> [--format text|json]`
- 前端：
  - `applyWorkspaceMessageAction(messageId: string, action: WorkspaceMessageAction): Promise<WorkspaceMessageActionResult>`
  - 浏览器预览必须基于当前内存 `WorkspaceBootstrapSnapshot` 本地回写，不能退回初始 seed

`WorkspaceMessageActionResult` 当前结构：

```json
{
  "action": "copy_code | open_link",
  "message_id": "msg_github_security",
  "copied_value": "362149",
  "opened_url": null,
  "snapshot": {}
}
```

动作语义：

- `copy_code` 只允许作用于存在 `extracted_code` 的消息
- `open_link` 只允许作用于存在 `verification_link` 的消息
- 动作执行成功后必须同时完成：
  - 将目标消息状态更新为 `processed`
  - 同步刷新 `message_groups`
  - 同步刷新 `message_details`
  - 同步刷新 `selected_message`
  - 同步刷新 `mailboxes.unread_count`
  - 同步刷新 `site_summaries.pending_count`
  - 从 `extracts` 中移除与本次动作匹配的条目
- `copy_code` 返回 `copied_value`，`opened_url` 必须为 `null`
- `open_link` 返回 `opened_url`，`copied_value` 必须为 `null`
- 前端详情面板按钮与 extract 卡片动作必须复用同一套 message action 合同，不能各自手写状态分叉

新增验证矩阵：

| 入口 | 条件 | 结果 |
|------|------|------|
| CLI `message action` | 缺少 `--id` | `InvalidCliArgs` |
| CLI `message action` | 缺少 `--action` | `InvalidCliArgs` |
| CLI `message action` | `--action` 非 `copy_code|open_link` | `Validation(field="action")` |
| Rust / Tauri / CLI `copy_code` | 目标消息无 `extracted_code` | `Validation(field="message.action")` |
| Rust / Tauri / CLI `open_link` | 目标消息无 `verification_link` | `Validation(field="message.action")` |
| 前端 `onExtractAction` | 未找到 extract 对应消息 | 显示可理解错误，不得静默吞掉 |

新增最小验证要求：

- `cargo test --manifest-path src-tauri/Cargo.toml`
  - 覆盖 `applies_copy_code_action_by_marking_processed_and_removing_extract`
  - 覆盖 `applies_open_link_action_by_marking_processed_and_removing_extract`
  - 覆盖 CLI `message_action_marks_processed_and_removes_matching_extract`
- `bun test`
  - 覆盖本地 `applyWorkspaceMessageAction`
  - 覆盖 `findWorkspaceMessageIdForExtract`
- `bun run build`
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- message action --id msg_github_security --action copy_code --format json`

### 6.9. M3 站点确认流补充合同（2026-04-05）

本轮补齐 M3 里“加入站点清单”的确认动作，要求 Rust 服务、Tauri command、CLI 与前端头部共享同一套站点确认语义。

- Rust / Tauri：
  - `confirm_workspace_site(input: String, label?: String) -> Result<WorkspaceBootstrapSnapshot, AppError>`
- CLI：
  - `site-context confirm --domain <domain> [--label <label>] [--format text|json]`
- 前端：
  - `confirmWorkspaceSite(input: string, label?: string | null): Promise<WorkspaceBootstrapSnapshot>`
  - 头部仅在“未命中且输入像完整域名”时展示确认动作

确认语义：

- 输入必须先经过与 `resolve_workspace_site_context` 相同的归一化流程
- 只有完整域名才允许确认加入；像 `lin` 这种不完整输入必须返回 `Validation(field="domain")`
- 已存在同 hostname 站点时，不允许重复新增；如果带了 `label`，允许更新 label
- 新增站点时必须写入 `site_summaries`
- 新增站点即使暂时没有任何消息命中，也必须保留在清单里，`pending_count = 0`
- 确认完成后前端应回写当前 snapshot，并将当前站点输入归一化为 hostname

新增验证矩阵：

| 入口 | 条件 | 结果 |
|------|------|------|
| CLI `site-context confirm` | 缺少 `--domain` | `InvalidCliArgs` |
| CLI / Tauri / 前端 | 输入无法归一化为完整域名 | `Validation(field="domain")` |
| CLI / Tauri / 前端 | 域名首次加入 | `site_summaries` 新增该 hostname |
| CLI / Tauri / 前端 | 域名已存在 | 更新 label 或直接返回现有项，不得重复追加 |

新增最小验证要求：

- `cargo test --manifest-path src-tauri/Cargo.toml site_context_confirm_adds_manual_site_to_snapshot`
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- site-context confirm --domain https://vercel.com/login --format json`
- `bun run build`

### 6.10. M3 独立已读状态补充合同（2026-04-05）

本轮补齐 M3 里“read/unread 独立于 pending/processed”的能力，要求 Rust 服务、Tauri command、CLI 与前端详情面板共享同一套已读状态语义。

- Rust / Tauri：
  - `update_workspace_message_read_state(message_id: String, read_state: MessageReadState) -> Result<WorkspaceBootstrapSnapshot, AppError>`
- CLI：
  - `message read-state --id <message-id> --state <unread|read> [--format text|json]`
- 前端：
  - `updateWorkspaceMessageReadState(messageId: string, readState: MessageReadState): Promise<WorkspaceBootstrapSnapshot>`
  - 详情面板必须提供“标记已读 / 标记未读”动作
  - 浏览器预览必须基于当前内存 `WorkspaceBootstrapSnapshot` 本地回写，不能退回初始 seed

已读状态语义：

- `read_state` 与 `status` 独立维护：
  - `read_state` 只表达是否已读
  - `status` 只表达是否已处理
- `message read-state` / `update_workspace_message_read_state` 只允许修改 `read_state`，不得隐式修改 `status`
- `message open` 与 `message original` 仍然会自动将目标消息置为 `read`
- `message mark --status processed` 与 `message action` 仍然会把目标消息一并置为 `read`
- 已读状态更新成功后必须同步刷新：
  - `message_groups[].items[].read_state`
  - `message_details[].read_state`
  - `selected_message.read_state`
  - `mailboxes.unread_count`
- 已读状态更新不应改变：
  - `message_groups` 的 pending / processed 归属
  - `site_summaries.pending_count`

新增验证矩阵：

| 入口 | 条件 | 结果 |
|------|------|------|
| CLI `message read-state` | 缺少 `--id` | `InvalidCliArgs` |
| CLI `message read-state` | 缺少 `--state` | `InvalidCliArgs` |
| CLI `message read-state` | `--state` 非 `unread|read` | `Validation(field="state")` |
| Rust / Tauri / CLI | 目标消息不存在 | `Validation(field="message.id")` |
| Rust / Tauri / CLI | 从 `unread -> read` | `unread_count` 递减，`status` 保持原值 |
| Rust / Tauri / CLI | 从 `read -> unread` | `unread_count` 递增，`status` 保持原值 |

新增最小验证要求：

- `cargo test --manifest-path src-tauri/Cargo.toml`
  - 覆盖 `updates_message_read_state_without_processing_message`
  - 覆盖 CLI `message_read_state_updates_snapshot_and_persists_read_flag`
- `bun test`
  - 覆盖本地 `applyWorkspaceMessageReadState`
  - 覆盖详情面板“标记已读”动作渲染
- `bun run build`
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- message read-state --id msg_github_security --state read --format json`

### 6.11. M4 新建邮件发送首切片补充合同（2026-04-06）

本轮正式启动 M4，但只交付“新建邮件发送”的第一条最小闭环；回复 / 转发和完整 SMTP 协议适配仍留到后续切片。当前要求 CLI、Tauri 与前端 Compose 面板共享同一套发送合同。

- Rust / Tauri：
  - `send_message(input: SendMessageInput) -> Result<SendMessageResult, AppError>`
- CLI：
  - `message send --account <account-id> --to <email> --subject <text> --body <text> [--format text|json]`
- 前端：
  - `sendMessage(input: SendMessageCommandInput): Promise<SendMessageResult>`
  - `ComposeWorkspace` 必须允许用户选择账号、输入收件人 / 主题 / 正文，并展示结构化发送反馈

`SendMessageInput` 当前合同：

```json
{
  "account_id": "acct_primary-example-com",
  "to": "dev@example.com",
  "subject": "Launch update",
  "body": "Shipping today."
}
```

`SendMessageResult` 当前合同：

```json
{
  "account_id": "acct_primary-example-com",
  "to": "dev@example.com",
  "subject": "Launch update",
  "status": "sent",
  "delivery_mode": "simulated",
  "summary": "已验证 acct_primary-example-com 的 SMTP 提交通道可达，并生成模拟发送回执。",
  "smtp_endpoint": "smtp.example.com:587"
}
```

当前阶段边界：

- `delivery_mode` 目前固定为 `simulated`
- 发送服务会复用：
  - 已保存账号元数据
  - 系统安全存储密码
  - SMTP 主机 / 端口配置
- 当前 live delivery 只验证 SMTP socket 可达并生成模拟发送回执，不冒充完整 SMTP 协议发送
- 浏览器预览模式不得伪造发送成功，必须明确提示仅桌面端可发送

新增验证矩阵：

| 入口 | 条件 | 结果 |
|------|------|------|
| CLI / Tauri / 前端 | `account_id` 为空 | `Validation(field="account_id")` |
| CLI / Tauri / 前端 | `to` 非法 | `Validation(field="to")` |
| CLI / Tauri / 前端 | `subject` 为空 | `Validation(field="subject")` |
| CLI / Tauri / 前端 | `body` 为空 | `Validation(field="body")` |
| CLI / Tauri / 前端 | 账号不存在 | `Validation(field="account_id")` |
| CLI / Tauri / 前端 | 系统安全存储缺少密码 | `Validation(field="account.credential")` |
| CLI / Tauri / 前端 | SMTP 提交通道不可达 | `Validation(field="smtp")` |

新增最小验证要求：

- `cargo test --manifest-path src-tauri/Cargo.toml`
  - 覆盖服务层 `sends_message_with_trimmed_input_and_resolved_account_credentials`
  - 覆盖 infra 层 `returns_simulated_receipt_when_smtp_socket_is_reachable`
  - 覆盖 CLI `message_send_returns_structured_json_result`
- `bun test`
  - 覆盖 `ComposeWorkspace` 渲染与 `buildSendMessageCommandInput`
- `bun run build`
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- message send --account acct_primary-example-com --to dev@example.com --subject "hello" --body "world" --format json`

### 6.12. M4 回复 / 转发预填流补充合同（2026-04-06）

本轮继续推进 M4，在“新建邮件发送”基础上补齐 reply / forward 的最小准备流。要求 CLI、Tauri 与前端本地回退共享同一套 compose 预填语义，而不是各自拼装不同草稿。

- Rust / Tauri：
  - `prepare_compose_draft(input: PrepareComposeInput) -> Result<PreparedComposeDraft, AppError>`
- CLI：
  - `compose prepare --mode <new|reply|forward> [--source-message <id>] [--account <id>] [--format text|json]`
- 前端：
  - `prepareComposeDraft(input: PrepareComposeCommandInput): Promise<PreparedComposeDraft>`
  - 浏览器预览模式必须复用本地 `prepareComposeDraftFromMessage(mode, message)`，与 Rust 侧保持同名字段和同等前缀 / 引用语义
  - 邮件详情面板必须提供“回复 / 转发”动作，并把用户带入同一个 Compose 面板，而不是打开另一套临时表单

`PrepareComposeInput` 当前合同：

```json
{
  "mode": "reply",
  "source_message_id": "msg_github_security",
  "account_id": null
}
```

`PreparedComposeDraft` 当前合同：

```json
{
  "mode": "reply",
  "account_id": "seed_primary-gmail",
  "to": "noreply@github.com",
  "subject": "Re: GitHub 安全验证码",
  "body": "\n\n在 2026-04-05T08:58:00Z，noreply@github.com 写道：\n> GitHub 安全验证码",
  "source_message_id": "msg_github_security"
}
```

预填语义：

- `mode = new`
  - 返回空草稿
  - 允许透传可选 `account_id`，便于 Compose 面板保留当前发件账号
- `mode = reply | forward`
  - 必须提供 `source_message_id`
  - 必须从同一份 workspace snapshot 读取来源邮件详情
  - `account_id` 以来源邮件所属账号为准，不允许前端另算
- `reply`
  - 默认 `to = source.sender`
  - `subject` 自动补 `Re: ` 前缀，但不得重复叠加
  - `body` 必须包含引用块，格式为 `在 <received_at>，<sender> 写道：` 加逐行 `> ` 引用
- `forward`
  - 默认 `to = ""`
  - `subject` 自动补 `Fwd: ` 前缀，但不得重复叠加
  - `body` 必须包含“转发邮件”引用头，并保留原发件人、账号、时间、主题与正文
- Compose 面板在 `reply | forward` 模式下必须显示：
  - 当前模式 badge
  - 来源邮件标题 / 发件人 / 账号 / 邮箱标签
  - “切回新建”动作

新增验证矩阵：

| 入口 | 条件 | 结果 |
|------|------|------|
| CLI / Tauri / 前端 | `mode = reply|forward` 但缺少 `source_message_id` | `Validation(field="source_message_id")` |
| CLI / Tauri / 前端 | `source_message_id` 不存在 | `Validation(field="message.id")` |
| CLI / Tauri / 前端 | 原主题已包含 `Re:` 或 `Fwd:` | 不得重复叠加主题前缀 |
| CLI / Tauri / 前端 | 浏览器预览模式点击回复 / 转发 | 必须走本地回退 helper，而不是伪造桌面端 invoke 成功 |

新增最小验证要求：

- `cargo test --manifest-path src-tauri/Cargo.toml`
  - 覆盖服务层 `prepares_reply_draft_from_workspace_message`
  - 覆盖服务层 `prepares_forward_draft_without_duplicate_prefix`
  - 覆盖服务层 `rejects_reply_prepare_without_source_message`
  - 覆盖 CLI `compose_prepare_returns_prefilled_reply_draft`
  - 覆盖 CLI `compose_prepare_returns_forward_draft_with_empty_recipient`
- `bun test`
  - 覆盖 `ComposeWorkspace` 的回复模式来源邮件展示
  - 覆盖前端本地 `prepareComposeDraftFromMessage()` 对 reply / forward 的前缀与引用语义
- `bun run build`
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- compose prepare --mode reply --source-message msg_github_security --format json`
