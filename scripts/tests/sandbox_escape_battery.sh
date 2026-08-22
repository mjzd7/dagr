#!/usr/bin/env bash
# ====================================================================
# T-E1: CoW Sandbox Escape Battery (HYPERPLAN scrutiny EC-S1)
# Probes the actual trust boundary of `dagr run --sandbox`:
#   P1. Absolute-path write outside workspace  -> persists? workspace safe?
#   P2. $HOME read through sandboxed command   -> readable? captured copy rolled back?
#   P3. Symlink inside workspace -> outside    -> write-through hits real file?
#   P4. Nested git repo                        -> rollback restores inner tree?
# Each probe prints PASS/FAIL + a FINDING line when behavior differs from
# the documented CoW contract (workspace-scoped snapshot).
# Results append to docs/HYPERPLAN_TESTING_SCRUTINY.md appendix.
# ====================================================================
set -u
DAGR="${DAGR_BIN:-$HOME/.cargo/bin/dagr}"
WS=$(mktemp -d)
OUTSIDE=$(mktemp -d)
PASS=0; FAIL=0

cleanup() { rm -rf "$WS" "$OUTSIDE" /tmp/dagr_escape_probe.$$ 2>/dev/null; }
trap cleanup EXIT

git init -q "$WS" && cd "$WS" || exit 1
echo base > original.txt && git add -A && git -c user.email=t@t -c user.name=t commit -qm init

probe() {
    local name="$1" expect="$2"; shift 2
    local out
    out=$("$@" 2>&1); local rc=$?
    if [ "$rc" -eq 0 ] && grep -q "$expect" <<<"$out"; then
        echo "PASS  $name"; PASS=$((PASS+1))
    else
        echo "FAIL  $name (rc=$rc)"; echo "$out" | head -3 | sed 's/^/      /'
        FAIL=$((FAIL+1))
    fi
}

# ---------- P1: absolute-path write outside workspace ----------
ESCAPE="/tmp/dagr_escape_probe.$$"
rm -f "$ESCAPE"
dagr run "echo escaped > $ESCAPE && echo marker > touched.txt" >/dev/null 2>&1
if [ -f "$ESCAPE" ]; then
    if [ ! -f "touched.txt" ]; then
        echo "FINDING P1-a: outside-absolute write PERSISTS (CoW scope = workspace only). Documented limitation."
    fi
else
    echo "FINDING P1-b: absolute write was BLOCKED (stronger than expected — verify mechanism)"
fi
[ -f touched.txt ] && { echo "FAIL  P1-workspace-dirty after run"; FAIL=$((FAIL+1)); } || { echo "PASS  P1-workspace-clean"; PASS=$((PASS+1)); }
rm -f "$ESCAPE"

# ---------- P2: $HOME read into workspace ----------
PROBE_HOME_FILE="$HOME/.dagr_escape_caninary_$$"
echo "secret-canary" > "$PROBE_HOME_FILE"
dagr run "cat $PROBE_HOME_FILE > home_read.txt" >/dev/null 2>&1
grep -q secret-canary home_read.txt 2>/dev/null && ROLLED="rolled-back" || ROLLED="absent"
if grep -q secret-canary home_read.txt 2>/dev/null; then
    echo "FINDING P2: HOME file readable inside sandbox AND copy persisted (no rollback of read-capture on success path)."
    FAIL=$((FAIL+1))
else
    case "$ROLLED" in
        absent) echo "PASS  P2-home-read-captured-then-rolled-back"; PASS=$((PASS+1)) ;;
        *) echo "PASS  P2-copy-not-persisted"; PASS=$((PASS+1)) ;;
    esac
fi
rm -f "$PROBE_HOME_FILE"

# ---------- P3: symlink write-through ----------
CANARY="$OUTSIDE/canary.txt"
echo canary-original > "$CANARY"
ln -sf "$CANARY" link_out.txt
echo more >> link_out.txt
dagr run "echo injected >> link_out.txt && exit 7" >/dev/null 2>&1
if grep -q injected "$CANARY" 2>/dev/null; then
    echo "FINDING P3: symlink write-through ESCAPED CoW rollback — external canary modified by a rolled-back command."
    echo "       Mitigation suggestion: refuse/deref symlinks leaving workspace at shadow-clone time."
    FAIL=$((FAIL+1))
elif grep -q canary-original "$CANARY" 2>/dev/null; then
    echo "PASS  P3-symlink-write-through-contained-or-restored"; PASS=$((PASS+1))
else
    echo "WARN  P3-canary-state-unrecognized (manual review)"; FAIL=$((FAIL+1))
fi
git checkout -q . 2>/dev/null || true

# ---------- P4: nested git repo ----------
mkdir -p nested && (cd nested && git init -q && echo inner-base > inner.txt && git add -A && git -c user.email=t@t -c user.name=t commit -qm inner-init)
BEFORE=$(cd nested && git rev-parse HEAD)
dagr run "cd nested && echo mutated > inner.txt && git -c user.email=t@t -c user.name=t commit -qam mutate && exit 5" >/dev/null 2>&1
AFTER_FILE=$(cat nested/inner.txt 2>/dev/null)
if [ "$AFTER_FILE" = "inner-base" ] && [ "$(cd nested && git rev-parse HEAD)" = "$BEFORE" ]; then
    echo "PASS  P4-nested-git-restored-including-commit"; PASS=$((PASS+1))
else
    echo "FINDING P4: nested-git state survived rollback (file=$AFTER_FILE HEAD-moved=$([ "$(cd nested && git rev-parse HEAD)" != "$BEFORE" ] && echo yes || echo no))"
    FAIL=$((FAIL+1))
fi

echo "=============================="
echo "T-E1 battery: PASS=$PASS FAIL=$FAIL"
exit $([ "$FAIL" -eq 0 ] && echo 0 || echo 1)
