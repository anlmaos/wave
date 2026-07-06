#!/usr/bin/env bash
# Scans TODO/FIXME/HACK/XXX comments in src/ and enforces a linking convention.
#
# Enforced formats:
#   TODO(<ISSUE-REF>): description
#   FIXME(<ISSUE-REF>): description
#   HACK(<ISSUE-REF>): description
#   XXX(<ISSUE-REF>): description
#
# Where <ISSUE-REF> matches one of:
#   #123
#   https://github.com/animaios/animawave/issues/123
#   animawave#123
#
# Bare TODOs (without a ref) are rejected. This signals the author to file an
# issue or link an existing one before merging.

export LC_ALL=C

# Two violation classes:
# 1. TODO/FIXME/HACK/XXX without an issue reference.
# 2. A TODO(…) with a paren but no URL or issue-ish token.

find_violations() {
  # Bare TODO: no parentheses, no issue link inline.
  grep -rEn '(//|#)\s*(TODO|FIXME|HACK|XXX)\s*:' src \
    --include='*.rs' --include='*.toml' --include='*.md'
  # TODO(…) with no URL and no #NNN pattern.
  grep -rEn '(//|#)\s*(TODO|FIXME|HACK|TODO)\s*\([^)]+\)' src \
    --include='*.rs' --include='*.toml' --include='*.md' \
    | grep -Ev 'https?://[^ )]+|#[0-9]+|/[a-zA-Z0-9_-]+#[0-9]+'
}

violations=$(find_violations | grep -v 'build-aux/check-tech-debt\.sh')

if [ -n "$violations" ]; then
  echo "Found TODO/FIXME/HACK/XXX comments without an issue reference:" >&2
  echo "$violations" >&2
  echo "" >&2
  echo "Format: TODO(#123): description" >&2
  echo "See https://github.com/animaios/animawave/issues to file one first." >&2
  exit 1
fi

echo "Tech-debt check OK (all TODO/FIXME/HACK/XXX comments are linked to issues)."
