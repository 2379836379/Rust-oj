# Rust-oj

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



