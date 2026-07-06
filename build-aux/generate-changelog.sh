#!/usr/bin/env bash
# Generate a CHANGELOG.md from git tags and commit subjects.
#
# Usage:
#   ./build-aux/generate-changelog.sh             # print to stdout
#   ./build-aux/generate-changelog.sh --write     # overwrite ./CHANGELOG.md
#
# Output format follows keepachangelog.com. Each git tag becomes a release
# section. Commit subjects are bucketed by conventional-commit prefix
# (feat: -> Added, fix: -> Fixed, docs: -> Documentation, ref: -> Changed,
#   everything else -> Changed).

set -euo pipefail
export LC_ALL=C

cd "$(dirname "$0")/.."

WRITE=0
[ "${1:-}" = "--write" ] && WRITE=1

bucket_added=""
bucket_fixed=""
bucket_changed=""
bucket_docs=""

add_commit() {
  local sha="$1" subject="$2"
  local msg="$subject"
  local bucket="changed"
  case "$msg" in
    feat:*) bucket="added"   ; msg="${msg#feat: }" ;;
    fix:*)  bucket="fixed"   ; msg="${msg#fix: }" ;;
    docs:*) bucket="docs"    ; msg="${msg#docs: }" ;;
    ref:*)  bucket="changed" ; msg="${msg#ref: }" ;;
    *)      bucket="changed" ;;
  esac
  msg=$(printf '%s' "$msg" | sed -E 's/^[space]+//; s/\.$//')
  msg="- ${msg} ([${sha:0:7}](https://github.com/animaios/animawave/commit/${sha}))"
  case "$bucket" in
    added) bucket_added="${bucket_added}${msg}\n" ;;
    fixed) bucket_fixed="${bucket_fixed}${msg}\n" ;;
    docs)  bucket_docs="${bucket_docs}${msg}\n" ;;
    *)     bucket_changed="${bucket_changed}${msg}\n" ;;
  esac
}

section_() {
  local title="$1" body="$2"
  if [ -n "$body" ]; then
    printf '### %s\n\n' "$title"
    printf '%b\n' "$body"
  fi
}

release_block() {
  local tag="$1"
  bucket_added=""; bucket_fixed=""; bucket_changed=""; bucket_docs=""

  local range
  if [ -n "${2:-}" ]; then
    range="${2}..${tag}"
  else
    range="${tag}"
  fi

  local date; date="$(git log -1 --format='%ai' "$tag" | cut -d' ' -f1)"
  local commits
  commits="$(git log --format='%H %s' "$range" || true)"

  if [ -n "$commits" ]; then
    while IFS= read -r line; do
      [ -z "$line" ] && continue
      sha="${line%% *}"
      subject="${line#* }"
      add_commit "$sha" "$subject"
    done <<< "$commits"
  fi

  printf '\n## [%s] - %s\n' "$tag" "$date"
  section_ "Added"          "$bucket_added"
  section_ "Fixed"          "$bucket_fixed"
  section_ "Changed"        "$bucket_changed"
  section_ "Documentation"  "$bucket_docs"
}

{
  printf '# Changelog\n\n'
  printf 'All notable changes to this project are documented in this file.\n'
  printf 'The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).\n\n'

  prev=""
  for tag in $(git tag -l --sort=-version:refname); do
    release_block "$tag" "${prev:-}"
    prev="$tag"
  done
} | if [ "$WRITE" -eq 1 ]; then
  cat > CHANGELOG.md
  echo "Wrote CHANGELOG.md"
else
  cat
fi
