# 实现 M1 实时连通性探测

## Goal

在已完成的 M1 账户 onboarding 首切片基础上，把 `account test` 从“规则预检”升级为“真实 socket 连通性探测”，让 CLI、Tauri Command 和前端都能基于同一套服务返回可观察、可验证的实时探测结果。

本轮不引入系统级安全存储，也不承诺完成完整 IMAP / SMTP 协议登录；重点是把“是否真的能打到目标主机和端口”这件事做实，并把文案语义从“预检”修正为“探测”。

## Context

- M0 已完成工作台底座、CLI 骨架与跨层共享链路。
- M1 首切片已完成账户元数据存储、账户表单与 `account add/list/test` 入口。
- 当前 `account test` 仍是基于端口与安全策略组合的规则判断，不会真的发起网络连接。
- 用户明确希望继续推进 M1，同时希望后续 Trellis 流程尽量避免“每个任务额外长一个 journal commit”。

## Decision

- 本轮 `account test` 定义为：先做输入校验与规则检查，再做真实 socket 连通性探测。
- 探测层级先到 TCP 连接成功 / 失败，不在本轮引入 TLS 握手、IMAP `CAPABILITY`、SMTP `EHLO` 等更深协议交互。
- 探测结果仍沿用现有 `AccountConnectionTestResult` 结构，但消息语义改为“实时探测”。
- 系统级安全存储继续留在下一切片，不在本轮用临时明文方案替代。

## Requirements

- `account test` 必须真的尝试连接 IMAP/SMTP 主机与端口。
- CLI 与 Tauri Command 必须继续复用同一套服务逻辑，不复制探测代码。
- 真实探测结果必须能区分：
  - 规则通过但实时连接失败
  - 规则可疑但实时连接成功
  - 主机不可达 / 超时 / 拒绝连接
- 前端 `Accounts` 视图文案必须同步从“预检”升级为“探测”。
- 需要保留现有账户新增与列表逻辑，不破坏 M1 首切片已交付能力。
- 需要把“单任务优先保持单个业务提交，不默认再生成额外 journal commit”的团队偏好写入 Trellis 规则。

## Acceptance Criteria

- [ ] `account test` 会发起真实 socket 连接探测
- [ ] Rust 服务层存在覆盖成功和失败场景的自动化测试
- [ ] CLI 层存在覆盖探测输出的自动化测试
- [ ] Tauri Command 仍只做薄装配
- [ ] 前端 `Accounts` 面板和结果文案改为“实时探测”语义
- [ ] 失败信息能明确告诉用户是规则问题还是连接问题
- [ ] `cargo test`
- [ ] `cargo fmt --check`
- [ ] `cargo clippy --all-targets --all-features`
- [ ] `bun test`
- [ ] `bun run build`

## Out Of Scope

- 系统级安全存储
- 密码持久化
- IMAP / SMTP 登录
- TLS 握手与证书校验
- 自动发现邮件服务配置

## Technical Notes

- 尽量使用标准库 `TcpStream::connect_timeout` 完成第一版探测，避免新增依赖阻塞本轮交付。
- 为了让测试稳定，优先在 Rust 测试中用本地 `TcpListener` 构造成功场景，用未监听端口构造失败场景。
- 如果需要区分规则检查与实时探测，可在服务层把两者合并为一个顺序执行的 tester。
