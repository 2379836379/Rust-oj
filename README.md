# oj-client (Rust + Tauri 重写版)

此目录用于在 `E:\Rust-oj\r` 下用 Rust + Tauri 重写 `QT/oj-client`（Qt Widgets 版）。

## 目标拆分（对应 Qt 目录）

- `QT/oj-client/src/config` → `src-tauri/src/config/*`：配置与运行状态（`config.toml` / `appstate.toml`）
- `QT/oj-client/src/network` → `src-tauri/src/network/*`：OpenJudge / OpenAI 访问（后续迁移）
- `QT/oj-client/src/parser` → `src-tauri/src/parser/*`：HTML/结果解析（后续迁移）
- `QT/oj-client/src/repository` → `src-tauri/src/repository/*`：SQLite/缓存（后续迁移）
- `QT/oj-client/src/service` → `src-tauri/src/service/*`：业务逻辑（后续迁移）
- `QT/oj-client/src/ui` → `src/*`：前端 UI（Tauri WebView）

## 已迁移（当前最小闭环）

- `config.toml`（OpenAI 配置）读写（优先兼容旧路径，默认写入用户配置目录）
- `appstate.toml`（提醒开关/铃声路径）读写（同上）
- 一个简单的设置页 UI：可编辑/保存上述配置

## 开发运行

前置：

- Rust toolchain（stable）
- Node.js 18+（或更新）

命令：

```powershell
cd r/oj-client
npm install
npm run tauri dev
```

## 迁移策略建议（后续）

1. 先把 “纯逻辑” 从 Qt 拆成 Rust 模块（网络、解析、存储、业务）
2. UI 只通过 Tauri `invoke` 调用 Rust 的 command（避免前端直接拼 URL/解析 HTML）
3. 每迁移一个页面（Login/Home/Class/Contest/Problem…），就补齐对应 service + parser + repository

