# Operations Runbooks

Short incident-response playbooks for the maintainers of Animawave. Each follows the same skeleton: **Detect → Diagnose → Mitigate → Record**.

---

## Runbook 1: GStreamer crash during playback

**Detect**: User reports segfault on station switch; `coredumpctl` shows `libgstreamer-1.0` stack frames. Often correlated with a specific codec/container fed by a malformed station URL.

**Diagnose**:
- Reproduce locally: `GST_DEBUG=4 flatpak run de.haeckerfelix.Animawave.Devel` — watch the GStreamer log for `gst_base_src_loop` error codes.
- Check recent commits that touched `audio/player.rs` or `audio/gstreamer_backend.rs` (the hotspots flagged in `AGENTS.md`).
- Verify whether the station URL still works in a standalone `gst-play-1.0 <url>` test.

**Mitigate**:
1. If a regression landed in the last N commits: `git revert <sha>` and push, file a follow-up bug.
2. If the station itself is broken: mark as invalid locally; report upstream to radio-browser.info (`/json/vote` can no longer recover a dead station).
3. Apply the reconnect-loop code path in `audio/player.rs::retry_stream` to handle transient failures without a crash.

**Record**: File an issue tagged `type:bug,area:audio,prio:high` and paste the GST log snippet. Open a separate upstream issue against GStreamer if you can reproduce with `gst-play-1.0` directly.

---

## Runbook 2: Flatpak build failure in CI

**Detect**: GitLab CI `flatpak@x86_64` job fails; common failure points listed in [AGENTS.md](AGENTS.md), table "Known Failure Modes".

**Diagnose**:
1. Fetch the failing job log from the GitLab pipeline page.
2. Search for "Build failure" — the line above it names the failing module/run.
3. Most often cause: `flatpak-builder --keep-build-dirs` reveals a new dep not present in `build-aux/de.haeckerfelix.Animawave.Devel.json`.

**Mitigate**:
- If manifest dep is missing: edit `build-aux/de.haeckerfelix.Animawave.Devel.json`, add the dep under `modules`, rebuild locally with `flatpak-builder --force-clean flatpak_app build-aux/de.haeckerfelix.Animawave.Devel.json`.
- If resolver/license change: `cargo deny` will surface it locally first. Pull the new `Cargo.lock`, run `cargo deny --log-level error check` from the sandbox.
- If flathub mirror is down: wait, do not push a mirror-bypass commit.

**Record**: Pin a CI issue with the exact manifest diff. Once landed, ship a reverted CI commit to green.

---

## Runbook 3: Radio-browser.info API outage

**Detect**: All searches return empty; `api/client.rs` logs HTTP 503 or timeouts. Third-party status pages (e.g. https://stats.uptimeradio-browser.info/) confirm the outage.

**Diagnose**:
- `curl -s 'https://de1.api.radio-browser.info/json/stations?limit=1'` — confirm the endpoint shape and response code.
- The `de1.api.radio-browser.info` subdomain may fail over to `at1.api.radio-browser.info` by convention; see `api/client.rs::API_BASE_URLS`.

**Mitigate**:
- Push a patch that rotates to the next mirror in `API_BASE_URLS` (already exists).
- Worst-case: keep local-library playback alive (SQLite cache, offline stations database). These don't require API access.

**Record**: Open an issue documenting the affected mirror; after the upstream recovers, revert the mirror pin.

---

## Runbook 4: Secret leak in PR

**Detect**: GitHub Secret Scanning alert; or a reviewer spots an API key / token in the diff.

**Mitigate** (do not wait):
1. Rotate the leaked secret immediately at its source (GitHub → Settings → Secrets).
2. Allow the PR merge. Do not amend commits to remove the secret — git history retains it.
3. Verify with `git log --all -p | grep '<prefix>'` that nothing else leaked.
4. Add `build-aux/check-tech-debt.sh`-style checks or pre-commit patterns to catch similar futures (e.g. gitleaks).

**Record**: Add to `.pre-commit-config.yaml` if a new scanner was needed; open a filed issue only if the leak was of a real secret (vs. test fixture) — do not cross-link to the commit SHA in the issue body.

---

## Runbook 5: i18n potfiles drift

**Detect**: `potfiles` CI job fails (`./build-aux/checks.sh potfiles`). Symptoms: `git diff --exit-code po/POTFILES.in` returns a non-empty diff.

**Diagnose**:
- Run `./build-aux/checks.sh potfiles` locally to regenerate.
- If diff is only cosmetic (e.g. additional filename), commit the new `POTFILES.in`.
- If `potfiles` misses a new source file: edit the glob list in `build-aux/checks.sh::potfiles`.

**Mitigate**:
- Commit the regenerated `POTFILES.in`.
- Land the fix and wait for Weblate (if used) to pickup the new pot template.

**Record**: Usually not issue-worthy; if your workflow is causing this repeatedly, document the commit-time hook (`pre-commit install` enforces this).

---

## Runbook 6: Post-deploy impact check

**Detect**: Deployment just landed on Flathub Beta or you pushed a new tag to `main`.

**Diagnose**:
- https://flathub.org/apps/de.haeckerfelix.Animawave.Devel — app page on Beta
- https://flathub.org/apps/de.haeckerfelix.Animawave — app page on Stable
- https://gitlab.gnome.org/haeckerfelix/animawave/-/pipelines — pipeline status
- https://github.com/animaios/animawave/actions — GitHub Actions status

**Verify**:
1. Check that the Flathub app page shows the correct new version.
2. Check that reviewers on the Flathub PR pass (manual step).
3. After merge, wait for Flathub build to finish (1–2 hours for Beta, 1–3 days for sync to Stable).
4. Install with `flatpak install flathub de.haeckerfelix.Animawave.Devel` and verify the new functionality.

**Record**: File an issue if the deploy stalled > 24 h; mention pipeline links.

---

## Useful URLs

| Surface | Location |
|---|---|
| CI pipelines | https://gitlab.gnome.org/haeckerfelix/animawave/-/pipelines |
| Flathub Beta listing | https://flathub.org/apps/de.haeckerfelix.Animawave.Devel |
| Radio-browser API status | https://wiki.radio-browser.info/ |
| Runtime image builds | https://quay.io/organization/gnome_infrastructure |
