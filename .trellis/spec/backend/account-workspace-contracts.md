# Account & Workspace Contracts

> 适用范围：`src-tauri/src/commands/account.rs`、`src-tauri/src/commands/workspace.rs`、`src-tauri/src/services/account_service.rs`、`src-tauri/src/services/workspace_service.rs`、`src-tauri/src/cli/mod.rs`、`src/lib/app-api.ts`、`src/components/account-form.ts`。

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

账户新增请求 `AddAccountInput`：

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

- `AccountConnectionTestInput` **没有** `password` 字段；当前探测只覆盖身份格式校验和 socket 可达性，不代表真实 IMAP / SMTP 认证成功。
- 前端表单允许 `login` 留空，但 [account-form.ts](../../../src/components/account-form.ts) 必须在发命令前自动回退为 `email`；Rust 侧命令契约仍要求收到非空 `login`。
- `AccountSummary.id` 由 `email` 归一化生成，格式为 `acct_<normalized-email>`，不能再回到基于序号的 ID。
- 账户元数据落盘路径优先级：`TWILL_ACCOUNT_STORE` 环境变量 > 平台应用数据目录下的 `Twill/accounts.json`。
- `accounts.json` 只保存元数据，**绝不能**写入明文密码；密码只进入系统安全存储。
- `list_accounts` 返回前必须根据系统安全存储结果重算 `credential_state`，不能盲信 JSON 内原始值。
- `add_account` 的顺序固定为：校验输入 -> 写元数据 -> 写系统密码 -> 若密码写入失败则回滚元数据。
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
| `add_account` | 元数据写入失败 | 返回 `Storage`，且不得写入系统密码 |
| `add_account` | 密码写入失败 | 返回原错误，且必须删除刚写入的元数据 |
| `list_accounts` / `add_account` | 文件锁等待超过 5 秒 | `Storage` |
| `test_account_connection` / `account test` | 输入校验失败 | 同上游 `Validation` |
| `test_account_connection` | 主机可解析但端口不可达 | `AccountConnectionTestResult.status = "failed"` |
| `test_account_connection` | 主机和端口可达 | `AccountConnectionTestResult.status = "passed"` |

### 5. Good / Base / Bad Cases

#### Good

- 前端留空 `login`，`buildAccountCommandInput()` 自动补成邮箱地址，再调用 `add_account`。
- `add_account` 成功后，`list_accounts` 返回的 `credential_state` 为 `stored`。
- `account test --format json` 返回 `status / summary / checks[]`，其中 `checks.target` 只允许 `identity | imap | smtp`。

#### Base

- 浏览器预览模式下可以渲染账号管理界面，但不应伪造系统密码保存能力。
- `test_account_connection` 只验证主机、端口和安全策略组合；它是“可达性探测”，不是“账号登录”。

#### Bad

- 让前端直接发送空 `login` 到 Rust 命令。
- 在 JSON 元数据中保存明文密码。
- 根据当前账户数量生成 `acct_1`、`acct_2` 这类易冲突 ID。
- 把 `test_account_connection` 当作真实邮箱登录成功的证明。

### 6. Tests Required

- `cargo test --manifest-path src-tauri/Cargo.toml`
  - 服务层：新增账号、重复邮箱、元数据失败、密码失败回滚、连接探测成功 / 失败。
  - 基础设施层：文件锁、并发保存、原子写。
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- account add ...`
  - 断言 CLI 和服务层共用同一合同。
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- account test ... --format json`
  - 断言 JSON 结构和成功 / 失败语义。
- `bun test`
  - 断言 [account-form.ts](../../../src/components/account-form.ts) 的 `login` 自动回退逻辑与 Rust 合同一致。

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

## 场景二：工作台快照加载与样例数据边界

### 1. Scope / Trigger

- 触发条件：修改 `load_workspace_bootstrap`、`workspace bootstrap` CLI、`WorkspaceBootstrapSnapshot` 结构、`src/data/workspace-bootstrap.json`。
- 触发条件：修改前端工作台对 `navigation / message_groups / extracts / site_summaries` 的消费方式。

### 2. Signatures

Rust：

```rust
#[tauri::command]
pub fn load_workspace_bootstrap() -> Result<WorkspaceBootstrapSnapshot, AppError>

pub fn load_workspace_bootstrap() -> WorkspaceBootstrapSnapshot
```

CLI：

```text
workspace bootstrap [--format text|json]
```

前端：

```ts
loadWorkspaceBootstrap(): Promise<WorkspaceBootstrapSnapshot>
```

### 3. Contracts

`WorkspaceBootstrapSnapshot` 当前字段：

```json
{
  "app_name": "Twill",
  "generated_at": "2026-04-05T00:00:00Z",
  "default_view": "recent_verification",
  "navigation": [],
  "message_groups": [],
  "selected_message": {},
  "extracts": [],
  "site_summaries": []
}
```

边界说明：

- 当前工作台邮件 / 验证码 / 网站摘要仍来自共享样例文件 [workspace-bootstrap.json](../../../src/data/workspace-bootstrap.json)。
- 共享样例文件必须同时被前端 import 和 Rust `include_str!` 消费，避免双份 mock 漂移。
- `loadWorkspaceBootstrap()` 在桌面端优先走 Tauri Command；不在 Tauri 环境或命令失败时，前端退回到打包内的同一份 JSON。
- `generated_at` 可以保留在快照结构里供调试 / CLI 使用，但默认用户界面不应把它当作产品信息展示。

### 4. Validation & Error Matrix

| 入口 | 条件 | 结果 |
|------|------|------|
| Rust `load_workspace_bootstrap` | JSON 非法 | 解析 panic，必须在测试 / 构建阶段暴露 |
| CLI `workspace bootstrap --format json` | `--format` 非 `text|json` | `UnsupportedFormat` |
| 前端 `loadWorkspaceBootstrap()` | 非桌面环境 | 直接返回打包快照 |
| 前端 `loadWorkspaceBootstrap()` | Tauri invoke 失败 | 返回打包快照，界面仍可加载 |

### 5. Good / Base / Bad Cases

#### Good

- 修改 `WorkspaceBootstrapSnapshot` 后，同时更新 Rust struct、TypeScript 类型和共享 JSON。
- 浏览器预览和桌面端看到同一份工作台样例结构。

#### Base

- 账号列表已经是真实数据，工作台邮件列表仍是样例快照；这是一条刻意保留的阶段性边界。

#### Bad

- 在前端单独维护另一份 `mockMessages`。
- 误把 `workspace bootstrap` 当成真实收件箱同步。
- 改了 Rust struct 但没改 `src/lib/app-types.ts` 或共享 JSON。

### 6. Tests Required

- `cargo test --manifest-path src-tauri/Cargo.toml`
  - 断言默认视图、导航项、消息组能从共享 JSON 成功加载。
- `cargo run --manifest-path src-tauri/Cargo.toml --bin twill-cli -- workspace bootstrap --format json`
  - 断言 CLI 输出的字段和 JSON 合同一致。
- `bun test`
  - 断言工作台分组、收件箱行渲染、未读图标和用户文案约束。

### 7. Wrong vs Correct

#### Wrong

- “先在 React 里手写一份 mock，Rust 那边以后再补。”
- “账号都是真实的，所以收件箱肯定也是真实的。”

#### Correct

- 共享样例快照只保留一份，前后端共用。
- 在产品说明和任务记录里明确：当前真实的是账号接入，不是真实邮件同步。
