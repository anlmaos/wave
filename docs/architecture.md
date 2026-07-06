# Animawave Architecture

Animawave is a GTK/libadwaita desktop internet-radio player for the [radio-browser.info](https://radio-browser.info/) catalogue. Build system is Meson with Rust; packaging is Flatpak (GNOME Nightly + Flathub).

## External dependencies

| System | Purpose |
|---|---|
| `radio-browser.info` HTTP JSON | Station search, vote, click tracking, tag browsing |
| `glycin` crate + loaders | Decoding station-icon PNG/JPEG from raw bytes |
| GStreamer (`gstreamer`, `gstreamer-audio`) | Playback pipeline: decodebin -> audioconvert -> pulsesink/pipewiresink |
| GSettings (`libadwaita::Preferences`) | User preferences, key bindings, window state |
| SQLite (`diesel` 2.x + bundled migrations) | Local station library, playback history |
| GUPnP (`cast-sender`) | Chromecast / network-radio device discovery |
| D-Bus MPRIS (`mpris-server`) | System media-keys, tray UI integration |

## Module topology

```mermaid
graph LR
    main --> app
    main --> i18n
    main --> path

    app --> ui
    app --> settings
    app --> audio
    app --> api

    ui --> settings
    ui --> audio
    i18n --> utils
    settings --> utils

    audio --> api
    audio --> database

    api --> utils

    database --> utils
    device --> ui
```

## Station-load flow

```mermaid
sequenceDiagram
    participant U as UI (search_page)
    participant A as api::client
    participant C as api::cover_loader
    participant Cache as cacache (disk)
    participant Radio as radio-browser.info
    participant Audio as audio::player
    participant GST as GStreamer pipeline

    U->>A: search(query="jazz")
    A->>Radio: GET /json/stations/byname/jazz
    Radio-->>A: Vec<Station>
    A-->>U: send stations

    U->>Audio: play(station)
    Audio->>A: client.click(uuid)           %% register listener-count
    Audio->>GST: set URI, play

    U->>C: load_cover(favicon_url)
    C->>Cache: read(cache_key)
    alt cache hit
        Cache-->>C: bytes
    else cache miss
        C->>Radio: GET favicon
        Cache->>C: write(bytes)
    end
    C-->>U: Paintable for station_cover
```

## Build and distribution paths

```mermaid
flowchart TB
    subgraph Source
        meson_build[meson.build]
        cargo[Cargo toolchain]
        flatpak_manifest[build-aux/Devel.json]
    end

    subgraph CI_Nightly[GitLab CI nightly]
        c_cargo[cargo-fmt, clippy, deny, typos]
        c_flatpak[flatpak-builder]
        c_publish[publish_nightly to flatpak repo]
    end

    subgraph CI_Docs[GitHub CI docs]
        changelog[generate-changelog.sh]
        cargo_doc[cargo doc --no-deps]
        agents_md_check[validate-agents-md.sh]
    end

    subgraph Dist
        nightly[Flathub Beta]
        stable[Flathub stable]
        gh_pages[gh-pages branch]
    end

    meson_build --> c_cargo
    cargo --> c_cargo
    c_cargo --> c_flatpak
    c_flatpak --> c_publish
    c_publish --> nightly

    meson_build --> changelog
    cargo --> cargo_doc
    cargo_doc --> gh_pages
```

## Data flow summary

```
User clicks station
  -> ui/event handler (channel::Sender<PlayerAction>)
    -> audio::player (GStreamer pipeline + recording)
      -> audio::gstreamer_backend (preroll, buffering, reconnect)
      -> database::library (last_played, history)
      -> device::cast_sender (optional — GUPnP Chromecast UI trigger)

Preferences change (GSettings notify)
  -> settings::settings_manager (Key enum, dbus proxy)
    -> audio_pipeline (reconfigure GST under the hood)
    -> ui (re-render SwitchRow / window state)
```

## Conventions

- Async tasks use `async-std` + `async-channel`. GStreamer callbacks must cross threads via `clone!(@weak obj => async move { ... })` — never touch GTK from a GStreamer bus callback.
- Database migrations live under `data/database/migrations/NNNN/`. New migrations are embedded at build time via `diesel_migrations::embed_migrations!`.
- Settings keys are defined once in `src/settings/settings_manager.rs::Key` (enum) and de/serialized into GSettings schemas under `data/`.

## Deployment observability

- Flathub Beta dashboard: https://flathub.org/apps/de.haeckerfelix.Animawave.Devel
- Flathub stable dashboard: https://flathub.org/apps/de.haeckerfelix.Animawave
- GitLab CI pipelines (Flatpak build): https://gitlab.gnome.org/haeckerfelix/animawave/-/pipelines
- GitHub Actions: https://github.com/animaios/animawave/actions

After a release tag is pushed here, the nightly job publishes to Flathub Beta. Promotion to stable is manual via the Flathub PR process — see RUNBOOKS.md.
