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
# Section-aware scan: only lines added INSIDE [dependencies],
# [dev-dependencies] or [build-dependencies] sections are new-dep
# candidates. Feature declarations (`foo = []`) and package metadata
# (`license =`, `description =`) live in other sections and must not
# trip the gate.
NEW_DEPS=$("${diff_cmd[@]}" --unified=0 -- 'Cargo.toml' '**/Cargo.toml' 2>/dev/null \
    | python3 -c '
import re, sys
DEP = re.compile(r"^\[(?:dev-|build-)?dependencies\]\s*$")
KEY = re.compile(r"^\s*[A-Za-z0-9_.-]+\s*=")
HUNK = re.compile(r"^@@ -\d+(?:,\d+)? \+(\d+)")
added, cur_file, pos = {}, None, 0
for line in sys.stdin.read().splitlines():
    if line.startswith("diff --git"):
        m = re.search(r" b/(.*)$", line)
        cur_file = m.group(1) if m else None
    elif line.startswith("@@"):
        m = HUNK.match(line)
        if m and cur_file:
            pos = int(m.group(1))
    elif cur_file and line.startswith("+"):
        added.setdefault(cur_file, {})[pos] = line[1:]
        pos += 1
    elif line.startswith("-") or line.startswith("\\"):
        continue
    else:
        pos += 1
bad = []
for f, lines in sorted(added.items()):
    try:
        content = open(f).read().splitlines()
    except OSError:
        continue
    sec, secmap = None, {}
    for i, l in enumerate(content, 1):
        st = l.strip()
        if st.startswith("["):
            sec = st
        secmap[i] = sec
    for i, l in sorted(lines.items()):
        if KEY.match(l) and DEP.match(secmap.get(i) or ""):
            bad.append(f"{f}: +{l.strip()}")
print("\n".join(bad))
' || true)

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
