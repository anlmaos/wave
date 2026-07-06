#!/usr/bin/env bash
# Disallow Rust files under src/ that exceed the line threshold.
# Current max is audio/player.rs (727 lines) so the 750 default lets existing
# code pass while catching further bloat in hotspot files.
export LC_ALL=C

THRESHOLD="${THRESHOLD:-750}"

violations=$(find src -name '*.rs' -exec wc -l {} + \
  | awk -v T="$THRESHOLD" '$1 > T && $2 != "total" {print "  " $2 " (" $1 " lines)"}')

if [ -n "$violations" ]; then
  echo "Files exceeding ${THRESHOLD} lines:" >&2
  echo "$violations" >&2
  exit 1
fi
echo "File-size check OK (all Rust files <= ${THRESHOLD} lines)."
