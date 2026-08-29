# Relay

Relay is a first-party-quality custom emulation console: a kiosk UI running on a mini PC that scans, identifies, and enriches a ROM library, then launches RetroArch/PCSX2/Dolphin to play it.

This is a **ground-up rewrite** of the original Electron MVP using **Tauri v2 + Rust**, with React kept for the UI. The rewrite is driven by footprint, performance, and Rust's fit for the systems-heavy parts of the app (file I/O, hashing, process management).

> Full execution plan and decision log: see `Tauri Rewrite` project in Linear (Relay team).

## Stack

| Layer | Technology |
|---|---|
| UI shell | Tauri v2 (native window, WebView) |
| Frontend | React + TypeScript, React Query for backend-driven state |
| Backend | Rust — single crate, modules (`db`, `ingestion`, `emulator`, `commands`) |
| Database | SQLite via `sqlx` |
| Emulator management | `tokio::process` |
| Metadata sources | IGDB, ScreenScraper |
| Kiosk / display | cage (Wayland compositor), Plymouth boot theme |
| Host | Beelink mini PC, Ubuntu Server, systemd-managed |

## Architecture at a glance

```
┌─────────────────────────────────────────┐
│  React UI (Tauri WebView)                │
│  React Query ↔ Tauri invoke/events        │
└───────────────┬───────────────────────────┘
                │ invoke / emit
┌───────────────▼───────────────────────────┐
│  Rust backend (single crate)              │
│  ├─ commands/   Tauri command handlers    │
│  ├─ db/         sqlx repositories         │
│  ├─ ingestion/  scan → probe → identify → enrich │
│  └─ emulator/   tokio::process management │
└───────────────┬───────────────────────────┘
                │
        ┌───────▼────────┐
        │  SQLite (sqlx)  │
        └─────────────────┘
```

## Ingestion pipeline

Four stages, each independently testable:

1. **Scan** — walk the ROM filesystem, produce a candidate file list
2. **Probe** — hash files, inspect headers
3. **Identify** — match against IGDB/ScreenScraper
4. **Enrich** — write metadata back, download images/assets

Progress is pushed to the UI live via Tauri events rather than polled.

## Development

```bash
# install deps
bun install        # or npm/pnpm, whichever the frontend package.json uses

# run in dev mode (hot-reloads React, restart required for Rust changes)
cargo tauri dev

# build a release binary
cargo tauri build
```

sqlx uses compile-time query checking, which needs a real SQLite file present at build time. Either keep a dev DB at the expected path or run:

```bash
cargo sqlx prepare
```

to generate offline query metadata for fresh clones / CI.

## Deployment

- Runs as a systemd service under cage (Wayland kiosk compositor)
- mDNS/Avahi exposes the device at `relay.local` for dev/deploy access
- Plymouth handles the boot splash

See the Phase 5 (System Integration) issues in Linear for setup details.

## Project status

This is a personal project, developed incrementally: a thin vertical slice first, then features layered in phase by phase, dogfooded on the actual Beelink hardware as it goes. See the `Tauri Rewrite` project in Linear for the full phase breakdown and current status.