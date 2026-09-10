# Relay

Relay is a self-built emulation console: custom hardware running a purpose-built
software stack, combining living-room gaming and movie playback (Jellyfin) on a
single box. This file orients Claude Code (or any contributor) to how the project
is structured and why.

## Architecture

Relay is split into a stable background daemon and a thin, disposable UI. The
daemon is the source of truth and the process owner for anything long-running;
the UI is just a renderer that can crash and restart without disrupting a game
in progress.

```
React UI  ─Tauri IPC─>  relay-app (thin Rust client)  ─Unix socket─>  relay-core (daemon)
                                                                            │
                                                              gamescope, BlueZ, NetworkManager,
                                                              emulator processes, CEC
```

- **`relay-core`** — the daemon. Runs as a system service from boot (systemd,
  `Restart=on-failure`). Owns all long-lived state and every subprocess it spawns
  (emulators included), so a UI crash never takes down a running game. Contains:
  - Game library (scanning, metadata, indexing)
  - Emulator manager (launch/monitor/kill, per-system backend selection)
  - Save manager
  - Controller manager
  - System manager
  - Power manager
  - CEC manager
  - Integrations: gamescope, BlueZ, NetworkManager
- **`relay-app`** — Tauri shell. Deliberately thin: no game/system logic lives
  here. Talks to the daemon over a Unix domain socket, renders state, forwards
  user intent as commands. On start (including after a crash), it requests a
  full state snapshot from the daemon, then subscribes to the daemon's event
  stream to stay in sync.
- **`relay-cli`** — command-line interface. Leaning toward this also being an
  IPC client of the daemon (for live state: what's running, library contents),
  rather than linking `relay-core` directly, so there's one source of truth.
  Direct-to-core "offline" commands (e.g. config validation) may be a reasonable
  exception when the daemon isn't running — still an open call.
- **`relay-protocol`** — shared, serde-typed request/response/event definitions
  used by daemon, CLI, and app, so the IPC contract is compiler-checked rather
  than hand-matched JSON across binaries.

### IPC

Unix domain socket, local only. Default to newline-delimited JSON over the
socket (`tokio` + `serde`) — simple, debuggable with `socat`/`nc`, no exotic
dependencies. Reconnect flow: client connects → requests full state snapshot →
subscribes to event stream for incremental updates.

### Why the split

The entire point is that killing/restarting the UI must never kill a running
game. That only holds if `relay-core` is the actual process supervisor for
emulators — not `relay-app`. Don't let emulator spawning creep back into the
UI layer.

Open question: if Relay itself ends up hosted as the gamescope session (instead
of launching under Steam's session), a UI crash may take the compositor session
down regardless of daemon state. Worth resolving whether the daemon or the UI
is the thing gamescope actually tracks as its session before this becomes load-bearing.

## Emulator strategy

Hybrid, not one-size-fits-all:

- **RetroArch / libretro cores** for 8/16/32-bit-ish systems (NES, SNES,
  Genesis, PS1, N64, GBA, etc.) — mature cores, unified controller abstraction,
  unified save states, one binary to manage.
- **Standalone emulators** where standalone development has clearly outpaced
  the libretro core: PCSX2 over the PS2 core, Dolphin standalone over its core,
  PPSSPP standalone for PSP, and similarly for anything heavier added later.

This means `emulator-manager` needs a real abstraction — an `EmulatorBackend`
trait with a common `launch(rom, profile) -> pid` interface. RetroArch is one
implementer (parameterized by core); each standalone emulator is another. Don't
special-case per-system logic outside this abstraction.

## Target platform

- **OS**: moving from Ubuntu to Arch Linux — minimal base, faster-moving
  `gamescope`/mesa/kernel, AUR tracks RetroArch/PCSX2/Dolphin upstream faster.
  Since the whole rework is about stability, treat the install as a pinned
  snapshot (Arch Linux Archive date), not rolling-release — only move the pin
  forward deliberately once a kernel/mesa/gamescope combo is verified good.
- **Hardware**: primary dedicated box is an HP EliteDesk 800 G3 Mini (35W,
  Intel HD Graphics 530/Skylake iGPU) — a real deployed target, not a dev
  stand-in. Also runs dual-boot (Relay/Windows) on a separate gaming PC with an
  RTX 3080.
- **Session/compositor**: cage/Wayland kiosk today; evaluating a custom
  gamescope session hosting Relay directly (instead of under Steam) for
  in-game overlay/menu support.
- **Networking**: NetworkManager (via `nmcli`) for network management; Samba +
  Avahi for guest-accessible LAN file shares (`roms`, `bios`, `saves`,
  `screenshots`).

## Stack

- Rust (Cargo workspace) for `relay-core`, `relay-app` backend, `relay-cli`,
  `relay-protocol`
- React + Tauri v2 for the UI
- No NAS / shared storage plan — single movable box, local storage. Cross-room
  "watch anywhere" for movies/TV is handled separately via off-the-shelf Apple
  TV clients, not a custom build.

## Conventions

- Favor lean, direct solutions. Push back on speculative abstraction — build
  the `EmulatorBackend` trait and similar boundaries because they're load-bearing
  for the hybrid emulator strategy, not by default elsewhere.
- This is a personal-use project, not a commercial product — optimize for
  George's actual hardware and workflow over generality.
- Treat premium consumer product aesthetics as the UI bar, even though it's a
  single-user tool.
