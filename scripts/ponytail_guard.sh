#!/usr/bin/env bash
# ====================================================================
# Ponytail Governance Guard (.ponytail.md contract)
#   ponytail_guard.sh          -> staged changes   (pre-commit mode)
#   ponytail_guard.sh <base>   -> diff <base>...HEAD  (CI mode)
#
# Checks:
#   FAIL  new dependency declarations in any Cargo.toml  (Ladder rung 5)
#   FAIL  container artifacts added                      (Prohibition #1)
#   WARN  diff larger than 400 changed lines             (Rung 6-7 nudge)
#
# Escape hatch for new deps: commit with --no-verify after ladder
# review and record a `// ponytail:` justification + PR note.
# ====================================================================
set -u

BASE="${1:-}"
if [ -n "$BASE" ]; then
    diff_cmd=(git diff "$BASE...HEAD")
    MODE="branch ($BASE...HEAD)"
else
    diff_cmd=(git diff --cached)
    MODE="staged"
fi

fail() { echo "❌ [PONYTAIL] $1" >&2; exit 1; }

echo "🥋 [PONYTAIL] Governance gate ($MODE)..."

# --- Ladder rung 5 / Prohibition on casual crates -------------------
NEW_DEPS=$("${diff_cmd[@]}" --unified=0 -- 'Cargo.toml' '**/Cargo.toml' 2>/dev/null \
    | grep -E '^\+[[:space:]]*[A-Za-z0-9_-]+[[:space:]]*=' \
    | grep -vE '^\+[[:space:]]*(version|features|optional|workspace|default-features|path|branch|tag|rev|package|registry|checksum|source)[[:space:]]*=' \
    || true)

if [ -n "$NEW_DEPS" ]; then
    {
        echo "$NEW_DEPS"
        echo ""
        echo "Ladder rung 5 requires stdlib / native platform / already-installed"
        echo "crates first. If genuinely unavoidable after ladder review, commit"
        echo "with --no-verify and add a '// ponytail:' justification."
    } >&2
    fail "New Cargo dependency detected."
fi

# --- Hard prohibition #1: containers in Phase 1 ---------------------
CONTAINERS=$("${diff_cmd[@]}" --name-only --diff-filter=d 2>/dev/null \
    | grep -Ei '(^|/)(Dockerfile[^/]*|docker-compose[^/]*\.ya?ml|compose\.ya?ml)$' \
    || true)

if [ -n "$CONTAINERS" ]; then
    echo "$CONTAINERS" >&2
    fail "Container artifact added — prohibited by .ponytail.md hard rule #1."
fi

# --- Rung 6-7 nudge: oversized change-sets (warn-only) --------------
CHANGED=$("${diff_cmd[@]}" --numstat 2>/dev/null | awk '{a+=$1; d+=$2} END {print a+d}')
if [ "${CHANGED:-0}" -gt 400 ]; then
    echo "⚠️  [PONYTAIL] WARN: $CHANGED changed lines — Ladder rung 1 asks whether all of it needs to exist." >&2
fi

echo "✅ [PONYTAIL] Contract upheld."
exit 0
