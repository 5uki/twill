# Directory Structure

> 后端目录目前很小，但不要因为规模小就把所有逻辑长期堆在 `lib.rs`。

---

## 当前结构

```text
src-tauri/
├── build.rs
├── Cargo.toml
├── tauri.conf.json
└── src/
    ├── main.rs
    └── lib.rs
```

---

## 当前职责

- `main.rs`
  - 应保持为启动入口。
  - 不承载复杂业务逻辑。

- `lib.rs`
  - 可以作为当前阶段的应用装配点。
  - 当命令和业务增长时，应拆分为更清晰的模块。

---

## 演进方向

当 `src-tauri/` 中开始出现多个功能模块时，优先按职责拆分：

```text
src-tauri/src/
├── main.rs
├── lib.rs
├── commands/
├── services/
├── domain/
├── infra/
└── cli/
```

### 拆分原则

- `commands/`：Tauri 对前端暴露的命令入口
- `services/`：应用服务，协调流程，不直接耦合 UI
- `domain/`：核心业务逻辑与领域模型
- `infra/`：文件、系统、外部依赖等实现
- `cli/`：CLI 模拟器入口

---

## 禁止事项

- 不把 UI 适配逻辑、系统 IO 和业务规则混写在同一个超大函数中。
- 不把临时调试逻辑长期留在启动入口。
- 不因为“先能跑”就无限推迟拆分；一旦出现第二个明显模块，就该开始抽层。
