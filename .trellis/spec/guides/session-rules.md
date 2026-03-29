# 会话规则

> 这份文档定义 Trellis 在 Asterway 项目里的会话入口约束。

---

## `/trellis:start` 是默认入口

- 开始新的开发会话时，默认先使用 `/trellis:start`。
- 如果会话开始时没有使用 `/trellis:start`，不要直接开始任务。
- 这时应先询问用户：是否明确选择不通过 `/trellis:start` 开始。

---

## 用户明确跳过时

如果用户明确表示跳过 `/trellis:start`，仍然要补做最小上下文检查：

1. 查看当前仓库结构和 git 状态
2. 确认当前任务或改动范围
3. 阅读对应 Trellis 规范入口
4. 再开始实现或分析

---

## 开发任务前的必读顺序

1. `workflow.md`
2. 对应层的 `index.md`
3. `guides/project-context.md`
4. `guides/project-engineering-baseline.md`
5. 与当前任务直接相关的具体规范文件

---

## 会话结束提醒

如果本次会话产生了代码或明确的任务推进，结束时应提醒用户：

- 运行 `/trellis:finish-work`
- 测试并提交代码
- 在提交后运行 `/trellis:record-session`
