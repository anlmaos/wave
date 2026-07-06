# AGENTS.md — Animawave Agent Protocol

## What Is Animawave?
Animawave is an open-source GTK4/libadwaita internet radio player for 50,000+ stations from radio-browser.info.

**Agent Readiness:** This file contains two kinds of guidance:
- **Universal rules** — project structure, branching, architecture, and security patterns that apply regardless of what tooling you have.
- **Workflow recommendations** — tool-specific tips that are helpful when the relevant tools are available, but not required to complete tasks. Use whatever tools you have access to.

## Repository Structure
```
animawave/
├── build-aux/          # Build scripts, Flatpak manifests
├── data/               # App resources (icons, UI, DB schema)
├── debian/             # Packaging metadata
├── po/                 # i18n files
├── src/
│   ├── api/            # Radio-browser API client
│   ├── audio/          # GStreamer backend, recording
│   ├── database/       # SQLite storage
│   ├── device/         # Device casting (Chromecast)
│   ├── settings/       # Configuration
│   └── ui/             # GTK widgets and dialogs
├── Cargo.toml          # Rust dependencies
├── meson.build         # Build system
└── README.md           # Docs
```

## Build, Test, and Run
```bash
# Flatpak (recommended)
flatpak install gnome-nightly de.haeckerfelix.Animawave.Devel

# Manual build
meson --prefix=/usr build
ninja -C build

# Quality checks
./build-aux/checks.sh cargo_fmt   # Format
./build-aux/checks.sh cargo_clippy # Lint
./build-aux/checks.sh cargo_deny   # Licensing
./build-aux/checks.sh potfiles     # i18n
```

## Quick start (single command)
```bash
meson setup build --profile=development && ninja -C build && ./build/src/animawave
```
Verify with: `./build-aux/checks.sh`;
run tests with: `cargo test --lib    # unit tests in src/
cargo test                         # + integration tests`

## Tooling Tips (Optional)

When available, structured analysis tools are more efficient than raw file reads:

- **Exploration**: Search for symbols, get file outlines, inspect call hierarchies
- **Impact analysis**: Check blast radius before making changes
- **Code discovery**: Use regex or text search for string literals
- **Build/test**: Use a terminal or shell at the project root

**Note**: Repeated raw file reads can be wasteful. Prefer targeted lookups when your toolset supports them.

---

## Code Exploration Policy

**Never use `read_file` or `grep` for exploration — they are editing tools only.**

For any "how does X work?", "where is Y defined?", or "find all Z" questions:
- **Symbols**: `search_symbols` (name/signature/summary), `get_call_hierarchy`, `find_references`
- **Structure**: `get_repo_map`, `get_tectonic_map`, `get_dependency_graph`
- **Impact**: `get_blast_radius`, `check_safe`, `plan_refactoring`
- **Strings/config**: `search_text` (string literals, comments, config values)
- **Patterns**: `search_ast` (anti-patterns, security issues, complexity)

**Only use `read_file` / `edit_file` when you have a symbol ID or exact file path and are making a change.**

---

## Session-Aware Routing (Confidence Tiers)

Every tool response includes `_meta.confidence` (0.0–1.0) and `reason`. Route your next action:

- **confidence >= 0.85** (High) — found exact match. Proceed; use result directly.
- **confidence 0.60–0.84** (Medium) — partial/fuzzy match. Verify with second tool (e.g. `search_text` to confirm, `get_context_bundle` for context).
- **confidence 0.35–0.59** (Low) — weak signal. Widen search: `fusion=true`, `semantic=true`, increase `token_budget`, try `mode="context"`.
- **confidence < 0.35** (Very low / no match) — **negative evidence**. Symbol likely doesn't exist or is named differently. Report gap, don't hallucinate.

**Negative evidence handling**: If `verdict: "no_implementation_found"`, `total_matches: 0`, or `confidence < 0.35`, treat as **confirmed absence**. Do not retry the same query. Instead:
1. Try broader query (`mode="context"`, no query string)
2. Search for related symbols via `get_blast_radius` on a known neighbor
3. Use `search_text` for string literals / comments
4. Report: "Symbol X not found in index (confidence Y). Tried: [queries]."

---

## Response Envelope Reading (Always Check `_meta`)

After every tool call, read `_meta` **before** the payload:

```json
"_meta": {
  "confidence": 0.72,
  "reason": "fuzzy_match_on_summary",
  "tokens_used": 3412,
  "tokens_remaining": 4588,
  "truncated": false,
  "total_matches": 0
}
```

**Mandatory checks:**
- `confidence < 0.6` → widen search or report gap (see Session-Aware Routing)
- `truncated: true` → increase `token_budget` or use `budget_strategy="compact"`
- `total_matches: 0` → negative evidence, not an error
- `reason` explains *why* — use it to choose next tool

---

## Model-Driven Tool Tiering

Match your model capability to tool choice:

- **Full tier** (Opus 4, Sonnet 4, GPT-4.1, Gemini 2.5 Pro): All tools, multi-step delegation, complex refactoring plans.
- **Standard tier** (Sonnet 3.7, GPT-4o, Gemini 2.0 Flash): `assemble_task_context`, `get_context_bundle`, `search_symbols`, `get_blast_radius`, `check_safe`. Avoid `plan_refactoring`, `embed_repo`, multi-agent orchestration.
- **Light tier** (Haiku, GPT-4o-mini, Flash-Lite): `search_symbols`, `get_repo_map`, `get_repo_health`, `read_file` (targeted). Avoid `assemble_task_context`, `get_context_bundle`, `get_call_hierarchy`.

**Rule**: If your model is Light/Standard tier, batch single-tool calls instead of complex multi-step workflows.

---

## ❗ Agent SOP — The Delegate-Verify Loop

**Follow every time. Never skip verification.**

### Step 1: Analyze & Plan
1. Map blast radius before changing — search symbols, check references
2. Identify integration points (`api/`, `audio/`, `ui/` interactions)
3. Break tasks into minimal, verifiable increments

### Step 2: Delegate One Step
- Pass complete context — the delegate or sub-agent is stateless and needs full context about repo structure, target symbols, and expected outcome
- Never bundle multiple changes into a single delegation

### Step 3: ❗ VERIFY THE RESULT
Agents frequently claim success while silently failing to edit files.

**After every delegated task:**
1. **Read the actual modified file** — confirm the expected code is present (e.g. via `grep` or reading file contents)
2. Check git status — `git --no-pager diff` before committing
3. Validate — verify downstream impact matches expectations (e.g. check references or call sites)
4. **Register edits** — update any indexes or caches your tooling requires
5. Test — run expected functionality plus adjacent components

> ❗ Lesson learned — CoverLoader: A subagent edited `api/cover_loader.rs` to fix cache eviction logic, then reported success. No edits were actually applied — the agent claimed it had updated the code when it had only read it. Only `read_file` confirmed the file was unchanged. Always verify the real file — never trust a delegate's self-report.

---

## Naming Conventions

Enforced linters + documented contracts ensure consistency regardless of author:

| Element | Convention | Example |
|---------|-----------|---------|
| Variables, functions, modules | `snake_case` | `format_duration`, `api::cover_loader` |
| Types, enums, traits, structs | `PascalCase` | `PlayerBufferStream`, `MediaExtractorError` |
| Constants, statics | `SCREAMING_SNAKE_CASE` | `MAX_RECONNECT_ATTEMPTS` |
| File names | `kebab-case.rs` | `gstreamer_backend.rs`, `cover_loader.rs` |
| GObject subclass public types | `Sw*` prefix | `SwApplicationWindow`, `SwTrackRow` |
| GObject private impl | `imp` module | `imp::SwApplicationWindow` |
| Test modules | `mod tests { #[cfg(test)] }` inside source file | `mod tests { use super::*; }` |

Violations are caught by `cargo clippy` (style lints) and rustfmt (formatting). Pre-commit hooks enforce both locally.

## Git Rules

### Branch Model
- **Base branch**: `main` — all PRs target this
- **Naming**: `feat/short-desc`, `fix/short-desc`, `docs/short-desc`
- **Sync**: `git pull origin main` before any commit

### Commit Format
```
<type>(<scope>): <message>

<optional body>

<optional footer>
```

| Type    | Scope examples          | Use case                          |
|---------|--------------------------|-----------------------------------|
| `feat`  | `audio`, `ui`, `api`      | New features, behavior changes    |
| `fix`   | `gstreamer`, `cast`       | Bug fixes, resilience improvements|
| `docs`  | `README`, `metainfo`      | Documentation updates             |
| `ref`   | `cleanup`, `sqlite`       | Refactoring without behavior change|

## Testing Rules

### How Tests Work
Animawave relies on CI gates enforced via `build-aux/checks.sh`:
- Format: `cargo fmt --all -- --check`
- Lint: `cargo clippy --all -- -D warnings`
- Licensing: `cargo deny --log-level error check`

**Run locally**: Always invoke via `checks.sh` wrapper.

### Known Failure Modes
| Failure                | Detection               | Recovery strategy          |
|------------------------|--------------------------|----------------------------|
| Non-empty potfiles diff| `./build-aux/checks.sh potfiles` | Add missing `src/*.rs` entries |
| CI formatting mismatch | Flatpak runtime fmt      | Rebase on `origin/main`, rerun |
| IANA private range     | `cargo deny`             | Patch resolver or exclude   |

## Architecture Landmines

| Component               | Hotspot                                          | Why it's risky                  |
|------------------------|-------------------------------------------------|----------------------------------|
| GStreamer backend      | `src/audio/gstreamer_backend.rs`                | Thread-sensitive callback timing  |
| Flatpak permissions    | `build-aux/*.json`                              | Missing `--share=network`         |
| Device casting         | `src/device/cast_sender.rs`                     | GUPnP context thread lifetime     |
| Recording mode         | `src/audio/recording_mode.rs` + GstClockTime    | File handle leaks, storage limits |
| CoverLoader cache      | `api/cover_loader.rs` + glycin crate            | `lcms2` format coercion crashes    |

> ❗ Hotspot example: GStreamer callbacks in `gstreamer_backend.rs` must never block or access GTK directly — use `clone!(@weak obj => async move { obj.do_gui_updates(); })` for thread transitions. Cross-thread violations cause runtime segfaults.

## Feature flags

Animawave uses GSettings-backed feature toggles — the GNOME-standard approach for desktop app configuration. The current toggles live in `src/settings/key.rs::Key`:

| Key | Type | Purpose |
|---|---|---|
| `buffer-duration` | u32 | GStreamer buffer size in seconds (set to `0` to disable buffering for low-latency) |
| `background-playback` | bool | Continue playing when the window is minimized |
| `notifications` | bool | Show desktop notifications for track changes |
| `recording-mode` | String | `auto`, `manual`, or `off` (toggle recording without code change) |

For development / nightly builds, use the `development` Meson profile via:
```bash
meson setup build --profile=development
```

This switches the app_id from `de.haeckerfelix.Animawave` to `Devel` and disables release-only feature gates. Never gate a feature behind a custom env var — use GSettings + a new entry in the Key enum instead. Pre-commit hooks prevent unreviewed toggles from reaching `main`.

## Credential Rules
This project has no deployment credentials.

**Safe import**: `.env.example` is checked in; `.env` is gitignored. For dev tokens (if ever needed), use:
```
cp .env.example .env
# Edit .env (containing placeholder values only)
```

## Service Dependencies

Animawave's external dependencies are documented in `docs/architecture.md`. Key systems:

| System | Purpose | Failure mode |
|---|---|---|
| `radio-browser.info` HTTP JSON | Station search, vote, click tracking | All searches return empty; use library cache |
| GStreamer (`gstreamer`, `gstreamer-audio`) | Playback pipeline | See RUNBOOKS.md — GStreamer crash runbook |
| glycin (image loader crate) | Station-icon PNG/JPEG decode | Canvas blank, `ERROR: glycin engine` |
| GSettings | User preferences | Defaults ignored if schemas aren't compiled |
| SQLite (`diesel` 2.x + embedded migrations) | Local library | Rebuild on schema-drift error |

When adding a new dependency, update `docs/architecture.md` and `AGENTS.md` tables in the same PR.

## Operations & Runbooks

Operational playbooks for maintainers are hosted in `RUNBOOKS.md`. Quick pointers:

| Problem | Page |
|---|---|
| GStreamer crash / segfault | [GStreamer crash runbook](RUNBOOKS.md/#runbook-1-gstreamer-crash-during-playback) |
| Flatpak build fails in CI | [Flatpak build failure runbook](RUNBOOKS.md/#runbook-2-flatpak-build-failure-in-ci) |
| Radio-browser.info API offline | [API outage runbook](RUNBOOKS.md/#runbook-3-radio-browser.info-api-outage) |
| Secret leaked in a commit | [Secret leak runbook](RUNBOOKS.md/#runbook-4-secret-leak-in-pr) |
| Potfiles drift fails CI | [i18n potfiles drift runbook](RUNBOOKS.md/#runbook-5-i18n-potfiles-drift) |

## UI/Component Gotchas

### GTK/libadwaita Traps

1. **Widget ownership** — GTK widgets exist in a singleton hierarchy. Mutating widgets created in a different context requires:
   ```rust
   clone!(@weak widget => move |_| {
       widget.set_property("visible", true);
   });
   ```

2. **GObject subclasses** — Must implement `Default`, have `#[glib::object_subclass]` macro, and must NEVER contain `
` in method signatures.

3. **Builder patterns** — Use `build_ui` helpers for *.ui template files:
   ```rust
   fn build_ui(&self) {
       let builder = gtk::Builder::from_resource("/path/to/ui/template.ui");
       let widget: CustomWidget = builder.object("name").unwrap();
   }
   ```

### Memory Traps
- `GString` from `String`: `.to_string()` -> `.into()`
- Boxed GDK types: `gdk::Rectangle::from(...)`
- Do not store `gtk::glib::WeakRef` across rust async blocks

---

## jcodemunch

Repo: `animaios/animawave` (indexed). Symbol ID: `{file_path}::{qualified_name}#{kind}`

### Core lookup
- `resolve_repo(path=".")` — **first call in a new workspace** — confirm repo is indexed; if not, run `index_folder(path="...")`
- `assemble_task_context(repo="animaios/animawave", task="...")` — opening move; auto-classifies intent (explore/debug/refactor/extend/audit/review), surfaces symbols + ranked context
- `get_file_outline` → `get_symbol_source` / `get_context_bundle(symbol_ids=[...])` — targeted retrieval, never full files
- `search_symbols(repo="animaios/animawave", query="...")` — find by name, signature, summary
  - `mode="context"` — query-less ranked context assembly
  - `mode="winnow"` — multi-axis constraint filter (kind, language, complexity, churn, etc.)
  - `semantic=true` — embedding-based search (requires embed provider)
  - `fusion=true` — Weighted Reciprocal Rank across lexical/structural/similarity/identity; best for vague queries
  - `detail_level="compact"` — 15 tokens/row for broad discovery; `"standard"` = signatures+summaries; `"full"` = inline source
- `search_text(repo="animaios/animawave", query="...")` — full-text search across file contents (string literals, comments, configs)
- `search_ast(repo="animaios/animawave", pattern="..." \| category="...")` — structural anti-pattern scan (empty_catch, god_function, hardcoded_secret, etc.)

### Impact & safety
- `get_blast_radius(symbol="...", depth=2, include_source=true, call_depth=0, include_decisions=false, source_budget=8000)` — check impact before changes; `call_depth>0` also finds callers; `include_decisions=true` surfaces git commit intent; `source_budget` caps snippet tokens
- `find_references(mode="refs"|"importers"|"related", quick=false)` — trace who uses a symbol; `quick=true` returns lightweight `{is_referenced, import_count, content_count}` for dead-code detection
- `get_call_hierarchy(symbol_id="...", direction="both", depth=3, chains=false)` — callers/callees N-deep; `chains=true` discovers HTTP routes, CLI commands, event signals; `kind` filters (http/cli/event/task/main/test)
- `check_safe(repo="animaios/animawave", symbol="...", mode="edit"|"delete")` — composite preflight: can this symbol be safely edited/deleted?
- `plan_refactoring(repo="animaios/animawave", symbol="...", refactor_type="rename"|"move"|"extract"|"signature")` — generate multi-file edit plan before refactoring
- `get_changed_symbols(repo="animaios/animawave")` — map git diff to affected symbols
- `get_pr_risk_profile(repo="animaios/animawave")` — unified risk assessment for a PR/branch

### Repository intelligence
- `get_repo_health(repo="animaios/animawave")` — one-call triage (dead code %, complexity, hotspots, cycle count)
- `get_repo_map(repo="animaios/animawave", mode="map"|"outline")` — signature-level overview ranked by PageRank (cold-start orientation); `mode="outline"` is a lightweight directory/language/symbol-count alternative
- `get_tectonic_map(repo="animaios/animawave")` — logical module topology (hidden boundaries, misplaced files, drifters)
- `find_hot_paths(repo="animaios/animawave")` — top-N symbols by runtime hit count (requires ingested traces)
- `get_dead_code_v2(repo="animaios/animawave", min_confidence=0.67)` — multi-signal dead code detection
- `find_similar_symbols(repo="animaios/animawave")` — cluster similar functions/methods (consolidation candidates)
- `get_symbol_provenance(repo="animaios/animawave", symbol="...")` — git authorship lineage & evolution narrative
- `get_symbol_complexity(repo="animaios/animawave", symbol_id="...")` — cyclomatic complexity, nesting, params
- `get_class_hierarchy(repo="animaios/animawave", class_name="...")` — inheritance ancestors + descendants
- `find_implementations(repo="animaios/animawave", symbol="...")` — find concrete impls of an interface/abstract
- `get_project_intel(repo="animaios/animawave")` — auto-discover Dockerfiles, CI configs, deps, APIs
- `list_workspaces(repo="animaios/animawave")` — enumerate monorepo workspace members
- `search_columns(repo="animaios/animawave", query="...")` — search column metadata across indexed models

### Runtime & indexing
- `import_runtime_signal(repo="animaios/animawave", path="...", source="otel"|"sql_log"|"stack_log")` — ingest runtime traces
- `embed_repo(repo="animaios/animawave", force=false, batch_size=50)` — precompute symbol embeddings for semantic search; `force=true` recomputes all embeddings
- `summarize_repo(repo="animaios/animawave", force=true)` — re-run AI summarization pipeline
- `index_file(path="...")` — surgical single-file reindex after edits
- `index_folder(path="...")` / `index_repo(url="...")` — full index/reindex
- `register_edit(repo="animaios/animawave", file_paths=[...], reindex=true)` — invalidate caches after file edits

### Power User Guide

#### Golden Rules
1. **Always start with `assemble_task_context`** — it auto-classifies intent and returns ranked symbols + context in one call. Never manually hunt for entry points.
2. **Batch everything** — use `symbol_ids[]` in `get_context_bundle`, `get_symbol_source`, `search_symbols` instead of serial calls. Token budget is your friend.
3. **Verify with `verify=true` / `verify_against="git_sha"`** — catches index drift vs. working tree.
4. **Use `mode` switches** on `search_symbols`: `context` for query-less ranked context, `winnow` for multi-axis filters, `semantic=true` for embedding search.
5. **Prefer `get_context_bundle` over raw file reads** — deduplicates imports, respects token budget, returns ready-to-use context.

#### Common Workflows

##### 1. Cold-start orientation (new repo / unfamiliar area)
```
resolve_repo(path=".")                                           # Confirm repo is indexed; if not → index_folder
get_repo_map(repo="animaios/animawave", group_by="flat", top_n=30)     # Top symbols by PageRank
get_tectonic_map(repo="animaios/animawave")                               # Logical module boundaries
get_repo_health(repo="animaios/animawave", detailed=true)                 # Dead code %, complexity, cycles
```

##### 2. Feature exploration — "How does X work?"
```
assemble_task_context(repo="animaios/animawave", task="How does X work?")
# → returns ranked symbols + context
get_context_bundle(symbol_ids=[...], budget_strategy="core_first")
```

##### 3. Refactoring safety (rename/move/extract)
```
check_safe(repo="animaios/animawave", symbol="SymbolName", mode="edit")
plan_refactoring(repo="animaios/animawave", symbol="SymbolName", refactor_type="rename", new_name="newName")
get_blast_radius(symbol="SymbolName", depth=2, include_source=true)
```

##### 4. Dead code cleanup
```
get_dead_code_v2(repo="animaios/animawave", min_confidence=0.67, file_pattern="src/**")
find_similar_symbols(repo="animaios/animawave", threshold=0.85, include_kinds=["function", "method"])
```

##### 5. Performance hotspot triage
```
find_hot_paths(repo="animaios/animawave", top_n=20)
get_repo_health(repo="animaios/animawave", detailed=true, top_n=30)
get_symbol_complexity(repo="animaios/animawave", symbol_id="...")
```

##### 6. PR / change risk assessment
```
get_changed_symbols(repo="animaios/animawave", include_blast_radius=true, max_blast_depth=3)
get_pr_risk_profile(repo="animaios/animawave", base_ref="main", head_ref="HEAD")
```

##### 7. Understanding unfamiliar code before modifying
```
get_symbol_provenance(repo="animaios/animawave", symbol="SymbolName", max_commits=30)
get_call_hierarchy(symbol_id="...", direction="both", depth=3, include_impact=true)
find_implementations(repo="animaios/animawave", symbol="InterfaceName", include_subclasses=true)
```

##### 8. Finding config / string literals / comments (not symbols)
```
search_text(repo="animaios/animawave", query="MAX_RETRIES", context_lines=3)
search_ast(repo="animaios/animawave", category="security")              # hardcoded_secret, eval_exec
search_ast(repo="animaios/animawave", pattern="string:/password/i")      # custom pattern
```

#### Parameter Cheatsheet

| Tool | Key params | Key combos | When to use |
|---|---|---|---|
| `resolve_repo` | `path` | `path="."` on startup | **First call in new workspace** — confirms index exists |
| `assemble_task_context` | `task`, `token_budget` (8k default) | `token_budget=4000` for focused tasks | **First call for any task** — returns intent, symbols, context |
| `search_symbols` | `mode`, `semantic`, `fusion`, `detail_level`, `token_budget` | `fusion=true, detail_level="compact"` for vague broad search | Symbol discovery; `mode=context` = ranked context w/o query |
| `get_context_bundle` | `symbol_ids[]`, `budget_strategy`, `token_budget` | `budget_strategy="compact"` for signatures-only; `"core_first"` to keep primary symbol | Multi-symbol context in one call |
| `get_blast_radius` | `depth`, `include_source`, `include_depth_scores`, `call_depth`, `include_decisions`, `source_budget` | `include_decisions=true, call_depth=1` for edit+call-chain risk | Pre-edit impact; `include_depth_scores` = per-hop risk |
| `find_references` | `mode` (refs/importers/related), `quick`, `cross_repo` | `mode="importers", quick=true` for dead-code shortcut | Trace imports, re-exports, callers |
| `get_call_hierarchy` | `direction`, `depth`, `chains`, `kind`, `max_depth`, `include_impact` | `chains=true, kind="http"` for route discovery; `include_impact=true` for deletion safety | Callers/callees/signal chains |
| `check_safe` | `mode` (edit/delete), `include_runtime` | `mode="edit"` before modifying signatures | Preflight — returns verdict + top-5 blockers |
| `plan_refactoring` | `refactor_type`, `new_name`/`new_file`/`new_signature` | — | Returns `{old_text, new_text}` blocks ready for Edit tool |
| `get_repo_health` | `detailed`, `rules` (layer defs) | `detailed=true` for full triage | One-call triage; `detailed=true` adds cycles, coupling, hotspots |
| `get_repo_map` | `group_by` (file/flat), `mode` (map/outline), `top_n` | `mode="outline"` for lightweight overview; `group_by="flat"` for ranked symbol list | Cold-start orientation |
| `get_tectonic_map` | `days`, `min_plate_size` | — | Module topology; finds drifters, nexus plates (coupled ≥4) |
| `find_similar_symbols` | `threshold`, `semantic_weight`, `include_tests` | `threshold=0.85, semantic_weight=0.6` | Consolidation candidates |
| `get_symbol_provenance` | `max_commits` | `max_commits=30` for deep history | Authorship lineage + evolution narrative |
| `search_ast` | `category`, `pattern`, `language` | `category="all"` for full sweep; `category="security"` for vulns | Anti-pattern sweep |
| `get_changed_symbols` | `since_sha`, `until_sha`, `include_blast_radius` | `include_blast_radius=true, max_blast_depth=3` for full impact | Maps git diff → symbols + downstream impact |
| `get_pr_risk_profile` | `base_ref`, `head_ref`, `days` | — | Composite risk (blast + complexity + churn + tests + volume) |
| `embed_repo` | `force`, `batch_size` | `force=true` to recompute all | Warm up embedding cache for semantic search |

#### Anti-patterns to Avoid
- ❌ Reading full files with `read_file` — use `get_context_bundle` or `get_symbol_source`
- ❌ Calling `search_symbols` repeatedly — batch with `symbol_ids[]` in `get_context_bundle`
- ❌ Skipping `check_safe` before edits/deletes — 5s call prevents hours of revert
- ❌ Not verifying with `verify=true` — index can drift from working tree
- ❌ Using `grep` for symbol lookup — `search_symbols` understands signatures, imports, types
- ❌ Manual blast radius tracing — `get_blast_radius(depth=2, include_source=true)` is instant
- ❌ Ignoring `_meta.confidence < 0.4` — low confidence means widen the search or report a gap, not proceed as-is
- ❌ Retrying the same query after `total_matches: 0` — that's confirmed absence, not a transient error

#### After Editing Files

Every file edit must be followed by:
1. `register_edit(repo="animaios/animawave", file_paths=["src/path/file.rs"], reindex=true)` — invalidate caches + reindex
2. Verify the edit landed (read the changed region or run `git --no-pager diff`)
3. Run adjacent tests or checks if applicable

#### Pro Tips
- **`fusion=true` on `search_symbols`** — uses Weighted Reciprocal Rank across lexical/structural/similarity/identity channels; best for vague queries
- **`budget_strategy="compact"`** on `get_context_bundle` — returns signatures only (min tokens), great for call-chain mapping
- **`include_decisions=true`** on `get_blast_radius` / `get_call_hierarchy(include_impact=true)` — surfaces git commit intent (revert/perf/refactor/bugfix) from history
- **`embed_repo(repo="animaios/animawave")` once** — then `semantic=true` on `search_symbols` works instantly for semantic queries
- **`index_file` after every edit** — keeps index fresh for subsequent tool calls in same session
- **`cross_repo=true`** on `get_blast_radius` / `find_references` — finds consumers in other indexed repos

#### Token Budget Discipline
- `assemble_task_context(token_budget=4000)` for focused tasks
- `get_context_bundle(token_budget=6000, budget_strategy="core_first")` for multi-symbol context
- `search_symbols(token_budget=3000)` with `detail_level="compact"` for broad discovery (15 tokens/row)
- Always check `_meta.tokens_used` / `_meta.tokens_remaining` in responses