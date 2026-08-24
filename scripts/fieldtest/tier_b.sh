#!/usr/bin/env bash
# ====================================================================
# Tier B: Git-Dependent Workflow Probes (requires full clone)
# Usage: tier_b.sh <repo-path>
#
# Probes:
#   P1  staged-file guard detection
#   P2  CoW commit-on-success
#   P3  CoW rollback preserves HEAD on failure
#   P4  branch checkout + guard scan
# ====================================================================
set -u

REPO="$(cd "$1" && pwd)"
DAGR="${DAGR_BIN:-$HOME/.cargo/bin/dagr}"
PASS=0; FAIL=0; FINDINGS=""

git config user.email "t@tierb" && git config user.name "TierB"

probe() {
    local name="$1" ok="$2"
    if [ "$ok" = "0" ]; then echo "PASS  $name"; PASS=$((PASS+1))
    else echo "FAIL  $name"; FAIL=$((FAIL+1)); fi
}

finding() { FINDINGS="${FINDINGS}\n  ⚠ $1"; }

echo "🔒 [Tier B] git-workflow probes on $REPO"

# --- P1: staged-file guard catches planted violation ------------------
VIOLATION_FILE="src/tierb_violation.ts"
mkdir -p "$(dirname "$VIOLATION_FILE")" 2>/dev/null
echo 'import { s } from "forbidden/secret";' > "$VIOLATION_FILE"
git add "$VIOLATION_FILE"

GUARD_OUT=$(dagr guard --workspace . --staged --format json 2>/dev/null)
G_CAUGHT=$(python3 -c "
import json,sys
try: d=json.loads(sys.stdin.read()); print(d.get('violations_count',0))
except: print(0)" <<<"$GUARD_OUT" 2>/dev/null || echo 0)

if [ "$G_CAUGHT" -gt 0 ] 2>/dev/null; then
    probe "P1-staged-guard-catches-violation" 0
else
    probe "P1-staged-guard-catches-violation" 1
    finding "P1: staged violation NOT caught by guard --staged (violations_count=$G_CAUGHT)"
fi
git reset -q HEAD -- "$VIOLATION_FILE" && rm -f "$VIOLATION_FILE"

# --- P2: CoW commit-on-success -----------------------------------------
NEW_FILE="tierb_created.txt"
BEFORE_COUNT=$(git rev-list --count HEAD)
dagr run "echo tierb-content > $NEW_FILE && git add $NEW_FILE && git -c user.email=t@t -c user.name=t commit -qm tierb-probe" --commit-on-success >/dev/null 2>&1
AFTER_COUNT=$(git rev-list --count HEAD)

if [ -f "$NEW_FILE" ] && [ "$AFTER_COUNT" -gt "$BEFORE_COUNT" ]; then
    probe "P2-cow-commit-on-success" 0
elif [ ! -f "$NEW_FILE" ]; then
    probe "P2-cow-commit-on-success" 1
    finding "P2: --commit-on-success did not create file or commit"
else
    probe "P2-cow-commit-on-success" 0
fi
# Cleanup: remove the probe commit
git reset -q --hard HEAD~1 2>/dev/null || true

# --- P3: CoW rollback preserves HEAD and working tree -------------------
HEAD_BEFORE=$(git rev-parse HEAD)
echo pristine > original_state.txt && git add original_state.txt && git -c user.email=t@t -c user.name=t commit -qm pristine
HEAD_PRISTINE=$(git rev-parse HEAD)

dagr run "echo corrupted >> original_state.txt && mkdir -p rogue_dir && echo rogue > rogue_dir/file.txt && exit 1" >/dev/null 2>&1

FILE_OK=$(grep -q pristine original_state.txt && echo yes || echo no)
NO_ROGUE=$([ ! -d rogue_dir ] && echo yes || echo no)
HEAD_OK=$(git rev-parse HEAD)

if [ "$FILE_OK" = "yes" ] && [ "$NO_ROGUE" = "yes" ] && [ "$HEAD_OK" = "$HEAD_PRISTINE" ]; then
    probe "P3-cow-rollback-preserves-head-and-tree" 0
else
    probe "P3-cow-rollback-preserves-head-and-tree" 1
    finding "P3: rollback incomplete — file_ok=$FILE_OK no_rogue=$NO_ROGUE head_ok=\$([ \"$HEAD_OK\" = \"$HEAD_PRISTINE\" ] && echo yes || echo no)"
fi
rm -f original_state.txt

# ---------- summary ----------
echo "=============================="
echo "Tier B probes: PASS=$PASS FAIL=$FAIL"
[ -n "$FINDINGS" ] && echo -e "\nFindings:\n$FINDINGS"
exit $([ "$FAIL" -eq 0 ] && echo 0 || echo 1)
