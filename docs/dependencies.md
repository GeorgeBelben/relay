# System dependencies

Packages `relay-core` assumes exist on the host but doesn't install itself.
Not scriptable/provisioning yet — just a running record so a fresh target
doesn't have to rediscover these one broken launch at a time. Update this
whenever a fresh install hits a "missing X" error.

Confirmed against Ubuntu 26.04 "resolute" (artemis.local). Arch package names
(for the EliteDesk, see [[REL-179]]) are not yet verified — check when that
migration actually happens rather than trusting the guesses below.

## Compositor

- `gamescope` — Ubuntu: `apt install gamescope` (universe). Arch: `gamescope`
  (extra). Relay routes every emulator through it; see
  `relay-core/src/gamescope.rs`.

## PCSX2 (PS2, standalone backend)

- `pcsx2` — Ubuntu: `apt install pcsx2` (multiverse). Arch (AUR): `pcsx2`.
- `libshaderc1` — Vulkan shader compiler. Without it PCSX2 fails with
  "Failed to compile utility pipelines" on first launch.

## RetroArch (libretro backend)

- `retroarch` — not yet installed on artemis; needed before REL-163's first
  real launch can happen there.
- Individual libretro cores, installed separately per system (e.g. `snes9x`)
  — package name and path vary by distro/core; `retroarch.core_path` is a
  required per-system setting rather than something relay-core discovers.

## Audio

- `pipewire`, `pipewire-pulse`, `wireplumber`, `pipewire-audio-client-libraries`
  — the client libraries alone (`libpipewire-0.3-0`, `libpulse0`, often pulled
  in transitively) are **not** sufficient. Without an actual running server,
  PCSX2's cubeb backend fails (`cubeb_stream_init() failed: CUBEB_ERROR (-1)`)
  and throws an on-screen error dialog that blocks the emulator view — looks
  like a frozen/stuck UI, not obviously an audio problem. Found on artemis,
  2026-09-16.

## Build-time (not just runtime)

- `libudev-dev` — needed to *compile* `relay-core`, not just run it. The
  `gilrs` crate (controller detection, REL-169) depends on `libudev-sys`,
  which shells out to `pkg-config` at build time looking for `libudev.pc`;
  without the `-dev` package that file doesn't exist and the build fails
  with `libudev-sys` panicking in its `build.rs`. Added to
  `.github/workflows/build-relay-core.yml`'s CI runner (`ubuntu-latest`
  ships `libudev1` but not `-dev`) 2026-09-16. Anyone building relay-core
  from source on a fresh Ubuntu box needs this too, not just CI.

## Unconfirmed, worth checking on the next fresh target

- `libxcb-cursor0` (Ubuntu) / `xcb-cursor0` (Arch) — Qt logged a warning
  wanting this during PCSX2 troubleshooting, but the actual failure that day
  was gamescope not running at all, not a missing package. Never explicitly
  confirmed present or absent as its own package — check if a fresh box hits
  a real Qt xcb-plugin load failure.
