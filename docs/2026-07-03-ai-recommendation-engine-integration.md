# AI Recommendation Engine Integration for Animawave

## Goal
Integrate an LLM-powered station recommendation engine that uses any OpenAI-compatible API. Users configure the endpoint in Preferences, right-click a station to get personalized suggestions based on listening history/favorites, then play or add recommended stations from a dialog.

## User-facing behavior

1. Preferences has a new **AI** page with three rows: Base URL (plain text entry), API key (password entry, Secret Service-backed with GSettings fallback), Model name (plain text entry).
2. Right-clicking a station in the **library grid**, **search grid**, or **player cover** shows a context menu with "AI recommend similar" when URL and model are non-empty.
3. Clicking it sends favorites + listening history to the configured LLM, parses the JSON response, and opens a `SwAiRecommendationsDialog` showing recommended stations.
4. Each row has **Play** (calls `player.set_station().await; player.start_playback().await`) and **Add to Library** (calls `library.add_station()`).
5. Failures degrade gracefully: empty config → menu hidden; network/auth/malformed error → `adw::Toast`, dialog opens empty; no crash, no alert friction.
6. If Secret Service is unavailable (no keyring daemon, Flatpak sandbox without `--talk-name=org.freedesktop.secrets`), API key falls back to plain GSettings string with a one-time-per-session warning toast.

## Architecture

### Component flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Animawave GTK4                                  │
│                                                                             │
│  ┌─────────────────┐   ┌──────────────────┐   ┌──────────────────────────┐ │
│  │ Preferences     │   │ Right-click on   │   │ SwAiRecommendationsDialog │ │
│  │ Dialog → AI pg  │   │ station grid     │   │                            │ │
│  │                 │   │                  │   │  List of stations          │ │
│  │ ┌─────────────┐ │   │ GestureClick    │   │  [Play] [Add to Library]  │ │
│  │ │base URL     │ │   │ (BUTTON_        │   │  "No suggestions" empty  │ │
│  │ │api key      │ │   │  SECONDARY)     │   │  state                    │ │
│  │ │model        │ │   │ gio::Menu        │   │                            │ │
│  │ └─────────────┘ │   │ "AI recommend"  │   │                            │ │
│  └────────┬────────┘   └────────┬─────────┘   └────────────┬───────────────┘ │
│           │                     │                            │               │
│           ▼                     ▼                            ▼               │
│  ┌────────────────┐   ┌─────────────────┐   ┌────────────────────────────┐ │
│  │ GSettings      │   │ src/api/llm/    │   │ SwStation constructed from │ │
│  │ + Secret Store │   │ LlmClient       │   │ response JSON              │ │
│  │ (key primary)  │   │ (reqwest)        │   │                            │ │
│  └────────────────┘   └────────┬─────────┘   └────────────────────────────┘ │
│                                │ POST                                         │
│                                ▼                                              │
│                    OpenAI-compatible endpoint                                 │
│                    /v1/chat/completions                                       │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Module layout

```
src/
  api/
    mod.rs                   — adds `pub mod llm;`
    llm/                     — NEW directory
      mod.rs                 — LlmClient struct (LazyLock<reqwest::Client>), recommend() entry
      recommend.rs           — POST to /v1/chat/completions, parse JSON
      prompt.rs              — SYSTEM_PROMPT const, build_context() from favorites+history
  settings/
    key.rs                   — adds AiBaseUrl, AiApiKey, AiModel variants
    secrets.rs               — NEW — store_api_key() / read_api_key() / StorageMode
    settings_manager.rs      — unchanged
  ui/
    mod.rs                   — adds `pub use ai_recommendations_dialog::SwAiRecommendationsDialog`
    display_error.rs         — unchanged, reused as-is
    preferences_dialog.rs    — adds AI PreferencesPage template child + secret-service integration
    preferences_dialog.ui    — adds embedded AI page (3 AdwEntryRows + info bar)
    ai_recommendations_dialog.rs   — NEW — adw::Dialog, ToastWindow impl, list of stations
    ai_recommendations_dialog.ui   — NEW — composite template
    pages/
      library_page.rs        — adds right-click GestureClick handler on grid
      search_page.rs         — adds right-click GestureClick handler on grid
    player/
      player_view.rs         — adds right-click handler on station_cover (button=GDK_BUTTON_SECONDARY)
      player_gadget.rs       — adds right-click handler on station_cover
  audio/
    player.rs                — unchanged (used for library context + play)
  database/
    library.rs               — unchanged (used for favorites read-only)

data/
  de.haeckerfelix.Animawave.gschema.xml.in  — adds 3 new keys
  gtk/
    preferences_dialog.ui                    — adds embedded AI page
    ai_recommendations_dialog.ui             — NEW — dialog template
```

### Storage model

Four new settings keys:
- `ai-base-url` (string, default "") — plain GSettings
- `ai-api-key` (string, default "") — dual-write: primary in Secret Service, GSettings as fallback
- `ai-model` (string, default "") — plain GSettings

No new SQLite schema, no new runtime services, no new ports.

### HTTP contract (raw reqwest, async-compat)

```
POST {ai-base-url}/v1/chat/completions
Authorization: Bearer {ai-api-key}
Content-Type: application/json

{
  "model": "{ai-model}",
  "response_format": {"type": "json_object"},
  "messages": [
    {"role": "system", "content": "<system prompt>"},
    {"role": "user",   "content": "<context built from favorites+history>"}
  ]
}
```

Provider returns a JSON array `[{"name": "...", "url": "..."}]`. We construct a `SwStation::new()` with the supplied URL. We fall back to string-parsing `choices[0].message.content` if `response_format: json_object` is unsupported.

### Prompt builder

Pulls up-to-N entries from:
- `SwApplication::default().library().model()` → `impl ListModel<Item=SwStation>`, read metadata name/tags/country
- `SwApplication::default().player().past_tracks()` → `SwTrackModel`, read `track.station().metadata().name` + `track.title()`

Produces a compact context string:

```
Favorites: Rock FM (rock, pop, DE), Jazz Wave (jazz, US)
Recently played: Night City Radio (electronic, synthwave, FI) — Morning Acoustic (folk, acoustic, SE)
```

System prompt: "You are a radio station recommender. Return a JSON array of objects with keys `name` and `url` (a real playable stream URL). Recommend 8-12 stations."

### Right-click pattern (new, not present in codebase today)

Each `Gtk::GridView` gets a `gtk::GestureClick` on its `ListItemFactory` (single-shot, button=GDK_BUTTON_SECONDARY). On press, build a `gio::Menu` with one entry → activate calls `window.activate_action("win.ai-recommend", Some(&station_variant))`. The action is installed at the window level only if URL and model are non-empty.

For `SwPlayerGadget` and `SwPlayerView`: attach a `GestureClick` to `SwStationCover` (which is a `gtk::Image` wrapped in a clickable box). The cover already displays the current station's icon — we compose a menu at click time using `player.station()`.

### Error handling

| Failure | Display |
|---|---|
| URL or model empty | Menu entry hidden (not sensitive) |
| Secret Service unavailable | Warning toast once per session; GSettings fallback used |
| Network/auth error (reqwest/4xx/toast) | `adw::Toast("AI recommendation failed: <reason>")`, dialog opens empty |
| Malformed JSON | Same toast, dialog opens empty |
| Empty valid response | Dialog opens with "No suggestions" info row |

Never logged: full API key (logged as `***REDACTED***`), full prompt (truncated 200 chars), full response (truncated 200 chars).

### Graceful degradation list

- Secret Service unavailable → fallback to GSettings, warning toast
- Provider doesn't support `response_format: json_object` → fall back to string-parsing
- Network error → toast, dialog opens empty
- Malformed JSON → toast, dialog opens empty
- Empty JSON list → dialog opens empty

### Flatpak manifest update

One new line in `build-aux/de.haeckerfelix.Animawave.Devel.json`:

```json
"--talk-name=org.freedesktop.secrets",
```

Already has `--share=network` so HTTP works.

## Milestones (execute-in-order, each leaves the product coherent)

### Milestone 1 — `settings`
- `src/settings/key.rs`: add AiBaseUrl, AiApiKey, AiModel
- `src/settings/secrets.rs`: `StorageMode` enum + `store_api_key` / `read_api_key` / `last_storage_mode` functions
- `data/de.haeckerfelix.Animawave.gschema.xml.in`: add 3 new keys
- `data/gtk/preferences_dialog.ui`: embed new AI PreferencesPage with 3 AdwEntryRows + info bar showing current StorageMode
- `src/ui/preferences_dialog.rs`: bind url/model via `settings_manager::bind_property`, integrate secret-service
- `Cargo.toml`: add `secret-service = "5.1"` with feature `rt-async-io-crypto-rust`
- Flatpak manifest: add `--talk-name=org.freedesktop.secrets`
- i18n: add translatable strings for labels/info bar

**Validation:** AI page renders; URL/model round-trip through `gsettings`; secret-service read/write path works; fallback path emits one warning toast; flatpak manifest still compiles; `flatpak-builder --run` builds successfully.

### Milestone 2 — `context-menu`
- `src/ui/window.rs`: install `win.ai-recommend` async action (only if URL and model non-empty)
- `src/ui/pages/library_page.rs`: add GestureClick (button=GDK_BUTTON_SECONDARY) on grid → build gio::Menu → activate `win.ai-recommend` with station variant
- `src/ui/pages/search_page.rs`: same pattern on search grid
- `src/ui/player/player_view.rs`: GestureClick on the station_cover area
- `src/ui/player/player_gadget.rs`: GestureClick on station_cover area

**Validation:** Right-click → "AI recommend similar" menu appears on all three surfaces; single-click activate still works; menu hidden when URL/model empty; no clippy warnings.

### Milestone 3 — `recommendations`
- `src/api/llm/mod.rs`: `LlmClient` struct + `LazyLock<reqwest::Client>` + `async-compat::Compat` wrapper; public `recommend(station)` entry
- `src/api/llm/recommend.rs`: HTTP POST, JSON parse, error mapping
- `src/api/llm/prompt.rs`: SYSTEM_PROMPT const, build_context function
- `src/api/mod.rs`: add `pub mod llm;`
- `src/ui/ai_recommendations_dialog.rs`: adw::Dialog + impl ToastWindow, list of stations with Play/Add buttons
- `data/gtk/ai_recommendations_dialog.ui`: composite template for the dialog
- `src/ui/mod.rs`: re-export new dialog
- `src/ui/window.rs`: route `win.ai-recommend` action → build dialog → present

**Validation:** Right-click + "AI recommend similar" against a real endpoint triggers the LLM call; dialog with recommended stations appears; Play starts playback; Add to Library persists; all failure modes degrade gracefully with toast; malformed JSON doesn't crash; one-time-per-session warning toast appears when using GSettings fallback.

## Risks & mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| secret-service rejected by Flathub | Build reject | Flag shipped early; Flathub talks to org.freedesktop.secrets literally every GNOME app ships — expected approval |
| Provider doesn't support `response_format: json_object` | Malformed reply | Try both; fall back to `choices[0].message.content` string parsing |
| Model returns hallucinated URLs | Unplayable station | HEAD request to verify; if it fails, try radio-browser.info search by name; if both fail, show "Unavailable" row with disabled Play |
| GNOME Keyring not running | Secret unreadable | GSettings fallback keeps feature working; warning toast |
| Network call blocks UI briefly | Flicker | `async-compat::Compat` keeps call off main thread |

## Constraints

- No new runtime service, port, or process.
- No new crate beyond `secret-service` (reqwest/serde/serde_json/async-compat already in project).
- No new SQLite schema; library table is read-only.
- No mock service for CI — unit tests cover prompt builder, parser, secrets fallback (no network needed); integration test uses mockito or similar for HTTP-level tests; user-facing test runs against user's real LLM endpoint.
- i18n: All user-visible strings wrapped in `i18n()` / `i18n_f()`; update `po/POTFILES.in` and `po/animawave.pot` in Milestone 3.
- All changes stay behind the existing `api::`, `ui::`, `settings::` module boundaries.