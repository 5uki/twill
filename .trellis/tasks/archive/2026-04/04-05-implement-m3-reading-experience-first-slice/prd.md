# 实现 M3 阅读体验首切片

## 目标

在 M2 已完成本地缓存、同步状态反馈和 mailbox/message CLI 查询基础上，启动 M3 的第一刀：

- 让工作台顶部搜索框真正可用
- 提供消息分类筛选
- 把邮件视图从“列表展示”升级为“列表 + 详情”的可阅读工作台
- 确保 CLI、Tauri Command 与前端桌面运行时遵循同一套消息筛选合同

## 本轮范围

### 交付

- `message list` 支持：
  - `--category <registration|security|marketing>`
  - `--query <keyword>`
- 新增 Tauri workspace 读命令：
  - `list_workspace_messages`
  - `read_workspace_message`
- 前端桌面运行时复用上述命令进行消息查询与详情读取
- 浏览器预览模式继续回退到共享种子快照，但沿用同名筛选字段与本地阅读辅助逻辑
- 工作台新增：
  - 顶部搜索
  - 分类筛选 chips
  - 列表/详情双栏
  - 详情中的验证码复制 / 验证链接打开动作
  - 空状态与查询失败提示

### 不在本轮

- 已读/未读状态持久化写回
- “标记已处理”命令与 UI 回写
- 站点候选确认流
- 原始邮件打开动作
- 更完整的 IMAP 阅读正文拉取

## 完成标准

- Rust 服务层与 CLI 测试覆盖 category/query 过滤
- 前端阅读辅助逻辑有 `bun test` 覆盖
- `bun test`、`bun run build`、`cargo test` 通过
- cross-layer 合同文档同步更新
