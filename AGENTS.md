# Egoroff.spb.ru Project Rules

## Project Overview
Personal website/blog with IndieAuth/Micropub support.

## Tech Stack
- **Backend**: Rust (workspace with 3 crates: egoroff, kernel, server)
- **Frontend**: Vue 3 + TypeScript + Bootstrap 5
- **Database**: SQLite
- **Build**: Cargo (Rust), Vue CLI (TypeScript)

## Architecture

### Hybrid UI
- **Public pages**: server-rendered Askama templates in `server` + HTML shells in `ui/public/`
- **Admin**: separate Vue SPA (`ui/src/AdminApp.vue`, hash-router in `ui/src/router/`)

### Crate graph
```
egoroff → server → kernel
```

### Key libraries
- **HTTP**: Axum, Tower middleware, `tower-sessions`
- **Templates**: Askama (`server/src/handlers/template.rs`)
- **Static assets**: `rust-embed` for built CSS/JS
- **Data**: SQLite via `kernel`

### Where to change what

| Task | Primary paths |
|------|---------------|
| Blog API / pages | `server/src/handlers/blog.rs`, `kernel/src/domain.rs` |
| Portfolio / downloads | `server/src/handlers/portfolio.rs`, `kernel/src/archive.rs` |
| IndieAuth / Micropub | `server/src/handlers/indie.rs`, `server/src/handlers/micropub.rs`, `server/src/auth.rs` |
| Admin REST API | `server/src/handlers/admin.rs`, `server/src/rest.rs` |
| Server templates | `server/src/handlers/template.rs`, `ui/public/*.html` |
| Admin UI | `ui/src/views/admin/`, `ui/src/components/admin/` |
| Frontend API client | `ui/src/services/ApiService.ts` |
| Frontend models | `ui/src/models/` |
| DB / storage logic | `kernel/src/sqlite.rs`, `kernel/src/domain.rs` |

## Project Structure
```
egoroff/
├── egoroff/      # Main CLI application
├── kernel/       # Core logic (SQLite, archive)
└── server/       # HTTP server (handlers, auth, micropub)
ui/               # Vue 3 frontend
```

## Coding Standards

### Rust
- Use `unsafe_code = "forbid"` (workspace lints)
- Prefer `Result<T, anyhow::Error>` for error handling
- Use async/await with tokio runtime
- Follow workspace dependencies versions
- Run `cargo clippy` before committing
- Create tests for a new functionality
- Write tests in AAA pattern
- If tests can be parameterized use `test-case` crate
- Code must pass all clippy pedantic validations
- Result code must be formatted using `cargo fmt` (`style_edition = "2024"`)
- Write code comments only in English

### TypeScript/Vue
- Use TypeScript strict mode
- Follow ESLint configuration (`eslint.config.mts`)
- Use Composition API for new components
- Import types explicitly
- Write code comments only in English
- UI strings and route names may be in Russian; code comments must be in English only

### Frontend layout
- Run `bun install` before first `bun run serve` or `bun run build`
- **Admin SPA**: Vue components under `ui/src/views/admin/` and `ui/src/components/admin/`
- **Public site**: mostly server templates + static HTML shells in `ui/public/` (not Vue SFCs)
- **API layer**: `ui/src/services/ApiService.ts` — keep in sync with `server` handlers
- **Types**: `ui/src/models/` (`blog.ts`, `dashboard.ts`, `common.ts`, etc.)
- `bun run devbuild` — development-mode frontend build (faster iteration)

## Commands

### Backend
```bash
cd egoroff
cargo build          # Build workspace
cargo test           # Run tests
cargo clippy --workspace -- -W clippy::pedantic  # Lint
cargo run -- server  # Start web server
```

### Frontend
```bash
cd ui
bun install          # First-time / after dependency changes
bun run serve        # Development server (admin)
bun run build        # Production build
bun run devbuild     # Development build
bun run lint         # ESLint check
```

## Important Notes
- Workspace uses resolver = "3" (Cargo 2024 edition)
- Release profile: LTO enabled, strip symbols, panic=abort
- Frontend uses esbuild-loader for optimization
- Frontend uses bun as package manager

## When Making Changes

### General
1. Check related files in the same module and cross-layer paths (see table above)
2. Ensure imports are updated
3. Run linters (`cargo clippy`, `bun run lint`)
4. Verify workspace compiles after changes

### By change type
- **Backend API** → `server/src/handlers/`, `kernel/src/domain.rs`, `ui/src/services/ApiService.ts`, `ui/src/models/`
- **New env var** → `server/src/lib.rs` (`ServerConfig::from_env`) and `README.md`
- **New admin page** → `ui/src/router/index.ts`, view under `ui/src/views/admin/`, matching handler in `server/src/handlers/admin.rs`
- **Public page / template** → Askama template in `server/src/handlers/template.rs`, HTML shell in `ui/public/`
- **Auth / sessions** → `server/src/auth.rs`, `kernel/src/session.rs`
- **Micropub / IndieWeb** → `server/src/handlers/micropub.rs`, `server/src/micropub.rs`, `server/src/handlers/indie.rs`

## Things NOT to do
- Do not use unsafe code
- Do not use `unwrap` or `expect` to get `Option` or `Result`
- Do not flag performance, copy/paste, or architecture nits in tests unless asked
- Do not suppress clippy warnings using attribute macros like `#[allow(clippy::unused_self)]`
- Do not write trivial code comments
- Do not write comments in Russian
