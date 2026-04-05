<!-- TRELLIS:START -->
# Trellis Instructions

These instructions are for AI assistants working in this project.

Use the `/trellis:start` command when starting a new session to:
- Initialize your developer identity
- Understand current project context
- Read relevant guidelines

Use `@/.trellis/` to learn:
- Development workflow (`workflow.md`)
- Project structure guidelines (`spec/`)
- Developer workspace (`workspace/`)

Keep this managed block so 'trellis update' can refresh the instructions.

<!-- TRELLIS:END -->

## 会话启动规则

- 新会话默认先使用 `/trellis:start`。
- 如果本次会话开始时没有使用 `/trellis:start`，不要直接开始开发任务。
- 这时应先询问用户：是否明确选择不通过 `/trellis:start` 开始。
- 只有在用户明确表示跳过后，才允许继续进入分析、实现或修改流程。
- 即使用户跳过 `/trellis:start`，也必须补做最小上下文检查：
  - 阅读 `.trellis/workflow.md`
  - 阅读相关 `.trellis/spec/` 入口文件
  - 检查当前任务、代码范围和上下文

## 项目硬规则

- Twill 是基于 `Tauri + React + TypeScript + Rust` 的多端邮箱客户端应用，UI 框架明确为 `React`。
- 所有思考、分析、解释和回答默认使用简体中文。
- 代码注释、技术文档、API 文档和用户说明默认使用中文。
- 严格遵循 TDD：先写失败测试，再写实现，再整理。
- 由于无法直接可靠操作 GUI，所有功能都必须提供 CLI 模拟器。
- 没有 CLI 模拟器的功能，默认视为未完成，不能算已交付。
- CLI 模拟器必须与 Tauri Command / 应用服务共用同一套核心逻辑，不允许复制业务逻辑。
- 修改功能时，不默认保留旧兼容代码；没有明确要求时，优先删除旧分支和旧协议。
- 避免过度防御性编程，优先保持代码简洁、清晰、可维护。
- 实现和评审时要关注算法复杂度、资源管理和边界场景。

## 编码与文本规则

- 所有源码、配置、脚本、Markdown、JSON 默认使用 `UTF-8` 编码；除非文件本身明确要求其他编码，否则不要改变编码。
- 修改包含中文或其他非 ASCII 内容的文件前，必须先确认文件编码；如果读取结果出现乱码，不要继续在乱码文本上增量编辑。
- 不要使用会隐式改变编码的默认文本读写方式，尤其是 PowerShell 的默认 `Get-Content` / `Set-Content` 行为。
- 如果必须使用 PowerShell 读写文本，必须显式指定 `-Encoding utf8`，并在写回后做最小验证。
- 发现乱码、私有区字符、异常引号、字符串字面量损坏、注释被破坏时，应立即停止继续修改，先修复编码问题或重建该文件的 UTF-8 正本。
- 不要对包含中文的大文件做未经验证的整段重写；重写前必须确认范围、来源和目标编码，重写后必须立刻做编译、测试或解析验证。
- 涉及字符串字面量、测试文案、错误文案的修改后，优先做一次最小编译或测试，确认没有因为编码问题引入语法错误。
- 如果 `.md`、`.json`、`.rs`、`.ts`、`.tsx` 文件出现编码异常，优先修复文件编码与文本完整性，再继续功能开发。

## 对话输出风格

- 默认输出环境按终端阅读优化，优先使用明显分组和留白。
- 复杂内容开头先给一句核心结论，再展开。
- 标题优先使用独占一行的 `**粗体**`，必要时可配少量 emoji 强化视觉锚点。
- 长段落拆成短句或短列表，多步骤任务使用 `1. 2. 3.` 有序列表。
- 复杂结构、流程或层级优先使用简洁的 ASCII 图示，并补一句说明。
- 多行代码、配置、日志必须使用带语言标识的 Markdown 代码块。
- 终端内容避免超长表格、超长路径和大段无分隔纯文字。
- 重点信息用 `**粗体**` 或 `*斜体*`，单行长度尽量控制在终端友好范围内。

## 提交偏好

- 如果用户明确要求“一个任务只保留一个最终提交”，AI 必须优先把 `.trellis/tasks/`、`.trellis/workspace/` 和相关说明文件一起准备到工作区中。
- 这种情况下，不要默认额外制造独立的 metadata commit 或 session commit。
- `record-session` 只作为记录流程参考；若用户要单最终提交，应改为手工更新记录文件并并入同一笔最终人工提交。
