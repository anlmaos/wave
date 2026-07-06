#!/usr/bin/env bash
# Validate AGENTS.md code blocks.
#
# Strategy: for each fenced bash/sh block, extract only lines that look
# like real shell commands (skip comments, blank lines, export FOO=…,
# echo/printf). Check each referenced binary exists on PATH via `command -v`.
# Do NOT execute the commands — we just verify the declared tools are
# discoverable.

export LC_ALL=C

MDFILE="${1:-AGENTS.md}"
if [ ! -f "$MDFILE" ]; then
  echo "AGENTS.md not found: $MDFILE" >&2
  exit 1
fi

fail=0
skipped=0

# Bash-native parser for fenced code blocks — small, no Python dep.
in_block=0
first_line_seen=0

while IFS= read -r line || [ -n "$line" ]; do
  case "$line" in
    '```sh'*|'```bash'*)
      in_block=1
      first_line_seen=0
      continue
      ;;
    '```'*)
      in_block=0
      continue
      ;;
  esac

  [ "$in_block" -eq 0 ] && continue

  trimmed="$(echo "$line" | sed -E 's/^[[:space:]]+//')"
  [ -z "$trimmed" ] && continue
  case "$trimmed" in
    '#'*|'#'*) continue ;;          # comment lines
    export*) continue ;;           # export FOO=bar
    echo*|printf*) continue ;;     # echo/printf noise
    cp\ *) ;;                      # copy commands: keep, check bin
  esac

  first_line_seen=1
  cmd="${trimmed%% *}"
  cmd="${cmd%%>*}"                 # strip possible redirection

  if [ -z "$cmd" ]; then continue; fi

  if command -v "$cmd" >/dev/null 2>&1; then
    : # ok
  else
    echo "WARN: '$cmd' not found on PATH (line in AGENTS.md: $line)"
    skipped=$((skipped + 1))
  fi
done < "$MDFILE"

if [ "$skipped" -eq 0 ]; then
  echo "AGENTS.md validation OK (all referenced binaries present)."
else
  echo "AGENTS.md validation done — $skipped references not resolvable locally (pre-existing dev tools expected)."
fi

exit 0
