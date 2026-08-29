# CLAUDE.md

Guidance for Claude Code when working in this repository.

## What this project is

Relay is a custom emulation console: a kiosk app that ingests a ROM library (scan/probe/identify/enrich), then launches emulators (RetroArch/PCSX2/Dolphin) to play games. It runs on a Beelink mini PC under Ubuntu Server + cage (Wayland kiosk compositor).

This repo is a **ground-up rewrite** from Electron to **Tauri v2 + Rust**, with React retained for the UI. The previous Electron MVP is a separate, archived codebase — do not assume its patterns carry over 1:1; several deliberately don't (see Decisions below).

## Tech stack

- **Shell**: Tauri v2
- **Frontend**: React + TypeScript
  - TanStack Router (file-based, `src/routes/**` → generated `src/routeTree.gen.ts`, gitignored) — memory history, not browser history (Tauri's webview isn't a plain http:// origin)
  - React Query for all backend-driven state (data from Rust commands)
  - Zustand or React Context for pure UI state only (view state, modals) — never route this through Tauri
  - Tauri's `emit`/`listen` event system for backend→frontend pushes (e.g. ingestion progress), not polling
  - Self-hosted fonts (`src/fonts.css` + `src/assets/fonts/`) — no CDN, this runs on a kiosk device with no guaranteed internet
  - `oxlint`/`oxfmt` for frontend linting/formatting (`bun run lint` / `bun run fmt`) — scoped away from `src-tauri/` via `.prettierignore`, since that's a separate Rust project formatted with `cargo fmt`
- **Backend**: Rust, single crate, module-based (not a Cargo workspace)
  - `src-tauri/src/db/` — sqlx repositories
  - `src-tauri/src/ingestion/` — scan → probe → identify → enrich pipeline
  - `src-tauri/src/emulator/` — `tokio::process` launch/kill/monitor
  - `src-tauri/src/commands/` — Tauri command handlers (thin, delegate to the above)
- **Database**: SQLite via `sqlx` (async, compile-time query checking)
- **Async runtime**: `tokio` throughout — keep DB and process handling consistently async

## Key decisions (don't relitigate these without discussion)

| Decision | Choice | Why |
|---|---|---|
| DB layer | sqlx, not diesel | Async-native, closer to the old Drizzle query-builder feel |
| Ingestion | Full Rust port, not a Node/Bun sidecar | Rust pays off most on I/O, hashing, parsing |
| Rust structure | Single crate + modules, not a workspace | No reuse/versioning need; modules are cheaper to reshape early on |
| State management | React Query + Tauri events for backend state | Backend is the source of truth for almost everything |
| Emulator management | Deep integration (crash detection, log capture), phased | Launch-and-forget feels like a regression for a "first-party" console |
| Migration | None — re-ingest test/real data fresh | No legacy DB to preserve |

## Build & dev commands

```bash
cargo tauri dev      # dev mode — React hot-reloads, Rust changes need a restart
cargo tauri build     # release build
cargo sqlx prepare    # regenerate offline query metadata (needed after schema changes, before commit)
cargo test            # Rust unit/integration tests
```

sqlx's `query!` macros check against a live DB at compile time. If you change the schema, run a migration locally, then `cargo sqlx prepare` before committing so CI and fresh clones don't need a live DB.

## Working conventions

- **Commands are thin.** Tauri command handlers in `commands/` should call into `db`/`ingestion`/`emulator` and map errors — business logic lives in those modules, not in the command layer.
- **Repository pattern for DB access.** Mirrors the pattern used in the Cove project — don't scatter raw `sqlx::query!` calls through command handlers.
- **Ingestion stages stay independently testable.** Each of scan/probe/identify/enrich should be callable and testable in isolation, not just as part of the full pipeline.
- **Emulator process handling uses `tokio::process`**, with exit codes and stdout/stderr captured — don't fire-and-forget a `Command::spawn`.
- **Don't reach for window-embedding solutions casually.** Embedding emulator windows inside the Tauri window is an open research spike (Phase 5) with a known fallback (cage-config focus management) — don't build features that assume it works until it's been validated.
- **Controller input**: RetroArch/PCSX2/Dolphin read controllers directly from the OS. Don't build an input-passthrough layer unless testing has actually shown a conflict with the Relay UI.

## What NOT to do

- Don't introduce a Cargo workspace / split crates without discussing first — this was a deliberate choice to avoid premature architecture.
- Don't add browser-storage-style persistence patterns (localStorage etc.) — this is a Tauri app; use the Rust/sqlx backend as the single source of truth.
- Don't silently change the DB layer to diesel or the async runtime away from tokio.
- Don't build migration tooling for the old Electron MVP's data — that data isn't being carried over.

## Where to look for more context

- Linear project **"Tauri Rewrite"** (Relay team) has the full phase-by-phase execution plan, with each task as its own issue attached to a milestone (Phase 0 through Phase 6).
- REL-59 tracks the Avahi/mDNS setup this rewrite needs to re-verify (Phase 5).