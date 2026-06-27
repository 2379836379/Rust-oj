# Rust-oj
## Project principles

- Implement features **in the same order as `QT/README.md`**, and regressively match the original Qt behavior.
- Do **not** add extra features beyond what exists in the Qt version.
- Keep the front-end page routing / pagination structure and layouts as close to the Qt project as possible.
- Use the same fixed endpoints as the Qt version for OJ-judger / OJ-server.
- Syntax highlighting: **Python only**.
- AI: **basic chat only** (no tool calling).

## Layout

- `src/`: Front-end UI (TypeScript). One file per page (login/home/class/contest/problem/favorite/storage/aiconfig).
- `src-tauri/`: Rust back-end (Tauri commands + network/parser/cache/storage).

Key modules:
- `src-tauri/src/state/`: `AppCtx` (session, command implementations, business orchestration)
- `src-tauri/src/network/`: OpenJudge HTTP + cookies
- `src-tauri/src/parser/`: HTML parsers (Class / Contest / Problem / Submit / Result)
- `src-tauri/src/cache/`: cache repositories (SQLite)
- `src-tauri/src/favorite/`: favorites (SQLite)
- `src-tauri/src/storage/`: storage stats/cleanup, login cache
- `src-tauri/src/config/`: `config.toml` / `appstate.toml`

## Development

Requirements:
- Node.js 18+
- Rust stable toolchain

Run (dev):
```powershell
cd r/oj-client
npm install
npm run tauri dev
```

Rust check:
```powershell
cd r/oj-client/src-tauri
cargo check
```

## Tech report

- See `r/oj-client/tech.md`

## Notes

- OpenJudge may redirect submission result pages to a `*.openjudge.cn` subdomain (e.g. `cxsjsx.openjudge.cn`).
 This merges root-domain cookies into subdomain requests (Qt `CookieStore::cookiesForUrl()` parity)
 so that result pages can be fetched and parsed correctly after submit.

