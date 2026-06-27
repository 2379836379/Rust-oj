# oj-client（Rust + Tauri 重写版）技术文档与实现报告

> 本文档面向本仓库 `r/oj-client` 下的 Rust + Tauri 实现。
> 目标是**按 `QT/README.md` 顺序**自回归复现 Qt Widgets 版本的功能；不做多余功能。

## 1. 项目目标与约束

### 1.1 重写目标
- 使用 **Rust + Tauri** 重写 `QT/oj-client`（Qt Widgets 版本）的 OpenJudge 桌面客户端。
- 前端页面结构（分页/布局/交互）尽可能与 Qt 版本一致。
- 逻辑尽量保持“Qt 行为一致”，特别是：登录/会话、刷新/自动刷新、提交结果闭环、缓存。

### 1.2 重要约束（来自需求）
- **按 `QT/README.md` 顺序推进**；实现源项目中的所有功能；不做额外功能。
- OJ-judger 与 OJ-server 地址使用与 Qt 版本一致的固定值（本项目内以常量形式存在）。
- 语法高亮：**只做 Python**。
- AI：只做最基础的对话（**不提供工具调用**）。

### 1.3 当前状态（闭环能力）
- 登录 + Home（课程/提醒）浏览
- Class / Contest / Problem 浏览
- Problem 页：本地 Test（调用 OJ-judger）+ Submit（调用 OpenJudge submitv2）+ Result 结果页解析与轮询
- Favorites、Storage、AI Config 等基础页
- Cookie 与缓存持久化（含跨 `*.openjudge.cn` 子域 cookie 合并）

## 2. 技术栈

- 前端：TypeScript + Vite（Tauri WebView 内运行）
- 后端：Rust（Tauri commands + 业务逻辑）
- 网络：reqwest + reqwest_cookie_store + cookie_store
- 解析：regex（兼容 Rust regex，无 look-around）
- 本地存储：
  - 配置：TOML（OpenAI config / AppState）
  - 缓存/收藏/登录缓存：SQLite（rusqlite bundled）+ 文件（cookies json）

## 3. 目录结构与模块边界

```
r/oj-client/
  src/                 # 前端 UI（TS），按“分页结构”实现
  src-tauri/
    src/
      main.rs          # Tauri command 注册入口
      lib.rs           # crate 根
      state/           # AppCtx：会话、业务聚合、命令实现
      network/         # OpenJudgeSession：HTTP + cookie
      parser/          # HTML/页面解析（Class/Contest/Problem/Submit/Result…）
      cache/           # 页面缓存仓库（SQLite/文件）
      storage/         # 登录缓存、存储占用统计、清理
      favorite/        # 收藏夹（SQLite）
      ai/              # 最基础对话（无工具）
      config/          # config.toml / appstate.toml
```

### 3.1 “Qt 对齐”的分层映射
- `QT/oj-client/src/network` → `src-tauri/src/network/*`
- `QT/oj-client/src/parser` → `src-tauri/src/parser/*`
- `QT/oj-client/src/repository`/`service` → Rust 中按功能折叠为 `state/ + cache/ + favorite/ + storage/`
- `QT/oj-client/src/ui/pages/*` → `src/ui/pages/*`（每个 page 一个 TS 文件）

### 3.2 设计原则
- UI 不直接解析 HTML：通过 `invoke()` 调 Rust command。
- Rust 层负责：网络请求、cookie、HTML 解析、缓存、统一错误信息。
- 解析失败不应导致应用崩溃：对 regex 初始化和解析结果做容错（避免 once_cell poison）。

## 4. 运行时架构与数据流

### 4.1 Tauri 命令边界
前端通过 `@tauri-apps/api/core` 的 `invoke()` 调用 Rust command。
入口：`r/oj-client/src-tauri/src/main.rs`

常用命令（非穷举）：
- 登录/会话：`oj_login`, `oj_logout`, `oj_get_joined_classes`
- 浏览：`oj_open_class`, `oj_open_contest`, `oj_open_problem`
- 提交闭环：`oj_open_submit`, `oj_submit_solution`, `oj_open_result`, `oj_result_is_waiting`
- 本地测试：`oj_judge_source`
- 收藏/存储：`oj_favorite_*`, `oj_storage_*`
- AI：`oj_ai_chat`（最基础）

### 4.2 AppCtx（后端全局状态）
核心：`r/oj-client/src-tauri/src/state/mod.rs`
- `OpenJudgeState`：
  - `base_url`（固定为 openjudge.cn）
  - `personal_home_url`（登录后解析到的用户主页）
  - `verified_email`（配合登录缓存/验证码逻辑）
  - `session: Option<OpenJudgeSession>`（**内存 session 优先**，对齐 Qt “一个 CookieJar”）
- cookie 持久化：`openjudge_cookies.json`（位于用户 config 目录）
- 登录缓存：`LoginCache`（SQLite/文件）

**关键点：**Rust 端优先复用内存 session；必要时从磁盘 cookie 重建 session。

### 4.3 网络层：OpenJudgeSession
核心：`r/oj-client/src-tauri/src/network/openjudge.rs`
- 使用 reqwest Client（启用 cookies、压缩、TLS）
- 默认 headers 对齐 Qt：User-Agent、Accept、Content-Type、Referer
- `get_html(url, referer)`：GET 并返回 (final_url, html)
- `post_form(url, body, referer, ajax)`：POST 表单，可选 `X-Requested-With: XMLHttpRequest`

#### 4.3.1 跨子域 cookie 合并（Qt 行为对齐）
Qt `CookieStore::cookiesForUrl()` 的逻辑：当请求 `*.openjudge.cn` 时，将 `openjudge.cn` 的 root cookies 一并带上。

本项目同样实现：
- 当目标 host 为 `*.openjudge.cn` 且不是 `openjudge.cn` 时：
  - 从 cookie store 中取 `openjudge.cn` 的 `get_request_values()`
  - 以 best-effort 插入到当前 url 的 cookie domain/path 匹配中

该修复用于解决：提交后跳转到类似 `http://cxsjsx.openjudge.cn/.../solution/<id>/` 的结果页时，因 cookie 未带上导致解析不到状态的问题。

### 4.4 登录与自动重登（Qt 行为对齐）
- `is_login_html()`：通过页面标题/关键字判断当前响应是否变成登录页。
- 当发现 session 过期：
  - 使用已保存的登录缓存（email/password）做一次自动重登
  - 再重试原请求一次

该逻辑用于：
- 浏览页面 GET
- 提交 POST（submitv2）

## 5. HTML 解析层（parser）

解析策略：
- 只使用 Rust `regex`（不支持 look-around），避免复杂正则；尽量使用“结构性锚点”。
- 尽量不依赖中文固定文本（例如“提交状态/状态:”），以适配多语言/站点差异。

### 5.1 Submit 页解析
文件：`r/oj-client/src-tauri/src/parser/submit.rs`
- 解析 `<form id="solution_submit" action="...">` 获取 submit action url
- 解析隐藏字段：`contestId`, `problemNumber`, `sourceEncode`
- 解析语言 radio 列表：`name=language` 的 `value/checked/label`

### 5.2 Submit payload 构造
- 对齐 Qt：source 使用 base64（与 `sourceEncode=base64` 配合）
- 使用 `application/x-www-form-urlencoded`

### 5.3 Result 页解析（提交状态）
文件：`r/oj-client/src-tauri/src/parser/result.rs`
支持解析你提供的结构，例如：
- `<div id="pageTitle"><h2>#52872585提交状态</h2></div>` → submission_id
- `<p class="compile-status"> ... <a href="..." class="result-wrong">Wrong Answer</a>`
  - solution_url / status_text / status_class

并提供：
- `is_waiting_status()`：Waiting 则前端/后端可轮询（Qt 2s）

> 注意：ResultParser 曾出现 regex 被污染导致 panic/poison 的问题；当前实现对 regex 构建做了容错（失败则用 `$^` 空匹配），避免应用崩溃。

## 6. Problem 页闭环（Test / Submit / Result）

前端文件：`r/oj-client/src/ui/pages/problem.ts`

### 6.1 本地 Test（OJ-judger）
- UI 侧选择语言（只做 Python 高亮；Test 可根据 label/value 推断 python/cpp，但需求上“高亮只做 python”）
- command：`oj_judge_source`
- 后端固定 judger 地址：`OJ_JUDGER_BASE_URL`（与 Qt 版本一致的固定值）

### 6.2 Submit（OpenJudge submitv2）
流程：
1) `oj_open_submit` 加载 SubmitPageInfo（语言/隐藏字段/action）
2) `oj_submit_solution` POST 到 `/api/solution/submitv2/`，携带 referer
3) 从响应 JSON 中取 `redirect`（Qt 行为）：得到结果页 URL
4) `oj_open_result` 打开结果页并解析状态
5) 若状态为 Waiting：2 秒后轮询刷新（Qt 行为）

#### 6.2.1 结果页 URL 的现实情况
OpenJudge 可能将结果页放在 `*.openjudge.cn` 子域，例如：
`http://cxsjsx.openjudge.cn/practise2026py/solution/52872585/`

因此必须保证：
- cookie 对子域可用（见 4.3.1）
- open_result 的 GET 请求带 referer（当前实现默认 referer 为 base_url）

## 7. 缓存与本地数据

### 7.1 cookies 持久化
- 文件：`openjudge_cookies.json`
- 写入时机：登录成功、关键请求后（提交后、重登后）
- 读取时机：启动/创建 session

### 7.2 登录缓存
- 用于：session 过期自动重登
- 存储：`LoginCache`（实现位于 `src-tauri/src/storage/*`）

### 7.3 页面缓存
- Class/Contest/Problem 的缓存仓库（SQLite）
- Home 页 due soon reminders 会优先读取缓存并与网络刷新结果合并
- Storage 页提供缓存大小统计与清理入口

## 8. 前端 UI 结构

### 8.1 Router
文件：`r/oj-client/src/ui/router.ts`
- 使用栈式路由（StackRouter），对齐 Qt “页面切换/返回”体验

### 8.2 Pages
`r/oj-client/src/ui/pages/*.ts`
- `login.ts`：登录
- `home.ts`：课程列表 + due soon reminders + refresh
- `class.ts`：课程页
- `contest.ts`：比赛页
- `problem.ts`：题目页（tools / submit / ai 三块与 Qt 对齐）
- `favorite.ts`：收藏
- `storage.ts`：缓存管理
- `aiconfig.ts`：AI 配置

### 8.3 语法高亮（仅 Python）
实现位置：Problem 页内部 `pythonHighlightHtml()`
- 轻量 tokenizer：关键字/数字/字符串/注释
- 不引入第三方高亮库，避免体积与复杂度

## 9. AI（最基础对话）
- command：`oj_ai_chat`
- 仅支持简单 messages → response
- 不实现工具调用（与需求一致）

## 10. 构建、运行与本地检查

### 10.1 环境要求
- Node.js 18+
- Rust stable toolchain

### 10.2 开发运行
```powershell
cd r/oj-client
npm install
npm run tauri dev
```

### 10.3 Rust 侧检查
```powershell
cd r/oj-client/src-tauri
cargo check
```

## 11. 调试与排障指南（常见问题）

### 11.1 提交后没有 Judge Status / 结果为空
现象：提交成功，但结果页解析不到状态。
排查顺序：
1) 确认 redirect 的结果页 URL 是否在 `*.openjudge.cn` 子域
2) 确认跨子域 cookie 合并逻辑是否生效
3) 打印 open_result 返回的 html_head（必要时临时加 debug command）

### 11.2 regex panic / once_cell poisoned
原因：
- Rust regex 不支持 look-around；或源码被错误脚本替换导致正则损坏。
策略：
- 避免使用 look-ahead/look-behind
- regex 初始化不要 `unwrap()` 直接 panic；失败时退化为空匹配
- 修改文件时确保 UTF-8，避免 Windows PowerShell 替换导致乱码

## 12. 与 Qt 版本对齐点（摘要）
- 请求头：UA/Accept/Referer 对齐 Qt `createRequest()`
- 提交：POST submitv2 带 `X-Requested-With: XMLHttpRequest`
- 解析 submitv2 回包：优先读取 JSON `redirect`（Qt 行为）
- Result：Waiting → 2 秒轮询刷新（Qt `ResultService`）
- Cookie：访问 `*.openjudge.cn` 时合并 `openjudge.cn` root cookies（Qt `CookieStore`）

## 13. 后续工作（按 QT/README.md 顺序）
> 本节仅用于记录推进顺序，不新增需求外功能。
- 继续对齐各页细节（布局、间距、拖动分隔、加载态一致性）
- 继续补齐剩余解析器与缓存策略（以 Qt 为准）
- 增强日志/诊断命令（仅用于对齐与排错，不做额外功能）

---

维护建议：
- 修改 Rust 源码时尽量使用 UTF-8 写入（避免脚本替换破坏编码）。
- 所有网络行为尽量通过 `OpenJudgeSession` 统一收口。
- Parser 尽量小步迭代，配合真实页面片段做回归。
