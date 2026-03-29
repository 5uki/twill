# UI 线框与结构定义

## 目标

基于已确认的 Fluent 工作台方向，给出一版可直接进入实现拆分的 UI 线框与结构定义。

适用范围：

- 首页工作台
- 最近验证邮件列表
- 右侧详情区
- 窄窗口退化策略
- 最小键盘交互基线

## 首页总线框

```text
+---------------------------------------------------------------------------------------------------+
| Top Bar                                                                                            |
| [Current Site Input................................] [Search..............] [Sync] [Account/Menu] |
+----------------------+-----------------------------------------------------+----------------------+
| Left Rail            | Center Pane                                         | Right Detail Pane    |
|                      |                                                     |                      |
| Twill                | Recent verification                                 | Quick Actions        |
|                      | [All] [Pending] [Has Code] [Has Link] [CurrentSite] | [Copy Code]          |
| > Recent verification|                                                     | [Open Link]          |
|   Unified inbox      | 09:41 GitHub        123456      admin@demo.com      | [Open Original]      |
|   By account         | 09:37 Notion        Open link   test@demo.com       |                      |
|   Sites              | 09:34 Stripe        884201      ops@demo.com        | Extracted Result     |
|                      | ---------------- processed ------------------------  | - Code               |
|                      | 09:30 Linear        used        admin@demo.com       | - Link               |
|                      |                                                     | - Site match         |
|                      |                                                     |                      |
|                      |                                                     | Original Message     |
|                      |                                                     | - Sender             |
|                      |                                                     | - Subject            |
|                      |                                                     | - HTML/Text body     |
+----------------------+-----------------------------------------------------+----------------------+
```

## 区域职责

### 顶栏

- `Current Site Input`
  - 输入或粘贴当前站点域名
  - 作为全局上下文影响列表筛选和站点匹配
- `Search`
  - 全局搜索邮件、站点、账号
  - 与站点上下文分离
- `Sync`
  - 手动刷新入口
  - 可显示最近同步状态
- `Account/Menu`
  - 多账号管理
  - 设置入口
  - 主题或应用菜单

### 左侧导航

- 一级只放产品视图，不放传统邮箱树
- 默认项：
  - `Recent verification`
  - `Unified inbox`
  - `By account`
  - `Sites`

### 中间主区

- 顶部显示当前视图标题
- 视图标题下方常驻常用筛选
- 主体使用高密度专业列表
- 默认排序为最新时间优先
- 已处理项沉到次级分组，不污染焦点区

### 右侧详情区

- 顶部固定快捷动作
- 中段展示提取结果
- 下段展示原始邮件内容
- 提取失败时，原始邮件区自动成为主要兜底路径

## 最近验证邮件列表结构

### 列表行线框

```text
+------------------------------------------------------------------------------------------------+
| 09:41 | GitHub | Code: 123456 | Link: No | admin@demo.com | Security | Pending | Subject text |
+------------------------------------------------------------------------------------------------+
```

### 建议字段顺序

1. 时间
2. 站点/平台名
3. 提取结果摘要
4. 账号归属
5. 分类标签
6. 处理状态
7. 次级主题信息

### 视觉层级

- 一级信息：
  - 时间
  - 站点名
  - 验证码或链接是否可用
- 二级信息：
  - 账号归属
  - 分类
  - 已处理状态
- 三级信息：
  - 主题文本
  - 发件人等辅助内容

### 列表行状态

- `Pending`
  - 默认高对比显示
- `Processed`
  - 移入次级分组
  - 在分组内弱化显示
- `Extraction failed`
  - 明确显示未提取成功
  - 引导用户看右侧原始邮件内容

## 右侧详情区结构

### 详情区线框

```text
+----------------------------------------------------------+
| Quick Actions                                            |
| [Copy Code] [Open Link] [Open Original] [Mark Processed] |
+----------------------------------------------------------+
| Extracted Result                                         |
| Site: github.com                                         |
| Code: 123456                                             |
| Link: https://...                                        |
| Match: Exact domain                                      |
| Confidence: Rule-based high                              |
+----------------------------------------------------------+
| Message Meta                                             |
| Account: admin@demo.com                                  |
| Sender: no-reply@github.com                              |
| Subject: Your verification code                          |
| Received: 2026-03-29 09:41                               |
+----------------------------------------------------------+
| Original Message                                         |
| HTML/Text preview...                                     |
+----------------------------------------------------------+
```

### 详情区规则

- 快捷动作固定在顶部，不随正文滚动丢失
- 提取结果默认展开
- 原始邮件内容默认可见，但在提取结果之后
- 当没有验证码、只有验证链接时：
  - `Copy Code` 置灰或隐藏
  - `Open Link` 成为主动作
- 当提取完全失败时：
  - 提取区显示失败原因或“未识别”
  - 原始邮件区提升为用户主要阅读区域

## 已处理项分组

### 默认列表

- 只聚焦待处理项
- 最新时间优先

### 次级分组

```text
Pending
- ...
- ...

Processed
- ...
- ...
```

规则：

- 执行高价值动作后可自动标记为已处理
- 用户可以手动撤销
- 已处理项保留回看能力，但不抢占默认焦点区

## 窄窗口退化

### 退化线框

```text
+----------------------------------------------------------------------------------+
| Top Bar                                                                           |
| [Current Site........................] [Search........] [Sync] [Account/Menu]    |
+----------------------+-----------------------------------------------------------+
| Left Rail            | Center Pane                                               |
|                      |                                                           |
| > Recent verification| 09:41 GitHub      123456     admin@demo.com              |
|   Unified inbox      | 09:37 Notion      Open link  test@demo.com               |
|   By account         |                                                           |
|   Sites              | [Open Detail Drawer]                                     |
+----------------------+-----------------------------------------------------------+
```

详情打开后：

- 右侧详情改为 `Drawer`
- 列表仍保持主上下文
- 关闭抽屉后返回列表，不切详情页

## 最小键盘交互基线

### 第一版建议快捷键

- `Up/Down`
  - 切换列表选中项
- `Enter`
  - 打开或聚焦详情
- `Ctrl/Cmd + C`
  - 在验证码存在时复制验证码
- `Ctrl/Cmd + Enter`
  - 打开验证链接
- `E`
  - 标记已处理或撤销
- `/`
  - 聚焦搜索
- `S`
  - 聚焦当前站点输入

### 原则

- 高频操作优先支持键盘
- 不要求第一版就做完整命令面板
- 所有快捷键都应有可见的鼠标入口

## Fluent 组件映射建议

### 顶栏

- `Toolbar`
- `Input`
- `Button`
- `Menu`
- `Avatar`

### 左侧导航

- 自定义 rail 容器
- `Button` / `TabList`
- `Badge`

### 列表与筛选

- `Button`
- `Badge`
- `Tag`
- 必要时评估 `Table` 或 `DataGrid`

### 右侧详情

- `Drawer`
- `Card`
- `Button`
- `Dialog`
- `Divider`

## 实现前建议拆分

1. 应用壳层与三栏布局
2. 顶栏站点上下文与搜索
3. `Recent verification` 列表与分组
4. 右侧详情与快捷动作
5. 窄窗口 `Drawer` 退化
6. 最小快捷键基线
