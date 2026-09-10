# Relay

Relay is a self-built emulation console — custom hardware and a
purpose-built software stack for living-room gaming and movie playback
(Jellyfin) on a single box, plus a controller-first UI for browsing and
launching your library.

## Features

- Unified library across emulated systems, scanned and indexed automatically
- Controller-first UI built for the couch, not a desktop
- Runs as a resilient background service — the UI can crash or restart
  without interrupting a game in progress
- Save management, system settings, and power/CEC control from one place
- Movie/TV playback via Jellyfin on the same box
- LAN file sharing (Samba + Avahi) for `roms`, `bios`, `saves`, and
  `screenshots`

## Hardware

- Primary box: HP EliteDesk 800 G3 Mini (35W, Intel HD Graphics
  530/Skylake iGPU)
- Also runs dual-boot (Relay/Windows) on a separate gaming PC with an
  RTX 3080
- Target OS: Arch Linux (pinned snapshot rather than rolling-release, for
  stability), running a cage/gamescope Wayland kiosk session

## Architecture

Relay is split into a stable background **daemon** and a thin, disposable
**UI**. The daemon is the source of truth and owns every long-running
process; the UI is just a renderer that can crash and restart without
disrupting a game in progress.

```
React UI  ─Tauri IPC─>  relay-app (thin Rust client)  ─Unix socket─>  relay-core (daemon)
                                                                            │
                                                              gamescope, BlueZ, NetworkManager,
                                                              emulator processes, CEC
```

**`relay-core`** is the daemon — it runs as a system service from boot and
survives independently of the UI. It owns:

- Game library (scanning, metadata, indexing)
- Emulator manager (launch/monitor/kill, per-system backend selection)
- Save manager
- Controller manager
- System manager
- Power manager
- CEC manager
- Integrations with gamescope, BlueZ, and NetworkManager

Critically, the daemon — not the UI — spawns and owns emulator processes.
That's what makes the crash guarantee real: if `relay-app` dies mid-game,
the emulator keeps running because it was never a child process of the UI
in the first place.

**`relay-app`** is the Tauri shell. It holds no game or system logic —
it talks to the daemon over a Unix domain socket, renders state, and
forwards user input as commands. On startup (including after a crash) it
requests a full state snapshot from the daemon, then subscribes to an
event stream to stay in sync.

**`relay-cli`** provides command-line access, most likely as another IPC
client of the daemon so there's a single source of truth for live state.

**`relay-protocol`** defines the shared, typed request/response/event
messages used across daemon, CLI, and app, so the IPC contract is
checked by the compiler rather than hand-matched JSON.

IPC itself runs over a local Unix domain socket using newline-delimited
JSON — simple, debuggable with standard tools, and fast enough for
commands and state sync (it's not carrying video).

### Emulator strategy

A hybrid approach rather than one engine for everything:

- **RetroArch / libretro cores** for 8/16/32-bit-ish systems (NES, SNES,
  Genesis, PS1, N64, GBA, etc.), for a unified controller abstraction and
  save-state format.
- **Standalone emulators** where they've clearly outpaced their libretro
  equivalents — PCSX2, Dolphin, PPSSPP, and similarly for anything heavier
  added later.

Each backend implements a common `EmulatorBackend` interface so the
emulator manager doesn't need per-system special-casing.

## Status

Actively in development. Currently reworking the architecture above —
pulling core logic out of the UI process and into `relay-core` as a
standalone daemon.
