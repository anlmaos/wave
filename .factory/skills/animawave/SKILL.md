---
name: animawave
description: Animawave GTK4/libadwaita internet radio player — project structure, build, architecture, and agent conventions.
---

# Animawave Skill

## What is Animawave?
GTK4/libadwaita desktop internet radio player for the radio-browser.info catalogue.

## Build & Run
```bash
meson setup build --profile=development && ninja -C build && ./build/src/animawave
```

## Quality Gates
```bash
./build-aux/checks.sh          # fmt + clippy + deny + typos + potfiles
./build-aux/checks.sh cargo_clippy
cargo test --lib               # unit tests
cargo test                     # + integration tests
```

## Architecture Overview
- `src/api/` — Radio-browser.info HTTP client (reqwest), cover loader (cacache)
- `src/audio/` — GStreamer backend, MPRIS, recording, playback state
- `src/database/` — Diesel 2.x + embedded SQLite migrations
- `src/device/` — GUPnP Chromecast sender
- `src/settings/` — GSettings keys (Key enum)
- `src/ui/` — GTK4/libadwaita widgets, template-based via `#[template(resource = ...)]`

## Key Conventions
- Naming: snake_case vars/fns, PascalCase types, SCREAMING_SNAKE_CASE consts
- File names: kebab-case.rs, GObject subclasses: Sw* prefix
- Async: `async-std` + `async-channel`; GStreamer callbacks must use `clone!(@weak obj => async move { ... })` — never touch GTK from a bus callback
- Tests: `#[cfg(test)] mod tests { use super::*; }` inside source files; integration tests in `tests/`
- i18n: `po/POTFILES.in` must stay in sync with source/UI files containing translatable strings

## Credential Rules
- `.env.example` is checked in; `.env` is gitignored.
- No deployment credentials.

## Hotspots
- `src/audio/gstreamer_backend.rs` — thread-sensitive callbacks
- `src/api/cover_loader.rs` — glycin + lcms2 format coercion crashes
- `src/audio/recording_mode.rs` — file handle leaks, storage limits
