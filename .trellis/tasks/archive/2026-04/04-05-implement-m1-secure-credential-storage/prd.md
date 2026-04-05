# 实现 M1 系统级凭据存储

## Goal

在已完成账户元数据保存与实时连通性探测的基础上，为 M1 补齐凭据输入与系统级安全存储链路，使账户 onboarding 真正具备“手动配置 + 安全保存”的最小闭环。

## Decision

- 账户密码不进入应用自己的 JSON 元数据文件，只进入系统安全存储。
- 运行时优先通过操作系统凭据存储保存密码；桌面端与 CLI 共用同一套 Rust 服务。
- 本轮只做单密码模型，不拆分 IMAP / SMTP 独立密码。
- `account test` 继续保留当前实时连通性探测职责，不在本轮扩成完整协议登录。

## Requirements

- `account add` 需要接收密码并写入系统安全存储。
- `account list` 需要返回凭据是否已存储的状态，但不能泄露任何敏感信息。
- 前端账户表单需要增加密码输入，并在保存成功后清空。
- CLI 需要支持密码参数，作为开发与回归验证入口。
- 自动化测试必须覆盖：
  - 成功保存元数据 + 成功保存凭据
  - 凭据写入失败
  - 列表状态正确反映凭据已存储 / 缺失

## Acceptance Criteria

- [ ] `AddAccountInput` 支持密码输入
- [ ] 系统安全存储通过独立基础设施层封装
- [ ] Tauri / CLI / 服务层复用同一套保存逻辑
- [ ] 账户列表可显示凭据状态
- [ ] 密码不会进入 JSON 元数据文件
- [ ] `cargo test`
- [ ] `cargo fmt --check`
- [ ] `cargo clippy --all-targets --all-features`
- [ ] `bun test`
- [ ] `bun run build`

## Out Of Scope

- IMAP / SMTP 协议登录
- 多密码模型
- OAuth
- 同步与收信
