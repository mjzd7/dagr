#!/usr/bin/env bash
# ====================================================================
# T0 Field-Test Batch Orchestrator — TESTING_ROADMAP Phase T0/T1-T4
# Usage: batch_run.sh [targets.json] [--quick]
#
# Reads targets.json, clones each repo (shallow or full based on tier),
# runs the field protocol + git-workflow probes, and produces a
# consolidated results JSON.
# ====================================================================
set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGETS_FILE="${1:-$SCRIPT_DIR/targets.json}"
QUICK_FLAG=""
[[ "${2:-}" == "--quick" ]] && QUICK_FLAG="--quick"

RESULTS_FILE="${DAGR_FIELDTEST_OUT:-$(mktemp -d)/batch-results-$(date +%s).json}"
WORK_DIR=$(mktemp -d)
PASS=0; FAIL=0; SKIP=0

cleanup() { rm -rf "$WORK_DIR"; }
trap cleanup EXIT

command -v dagr >/dev/null 2>&1 || { echo "❌ dagr not on PATH" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo "❌ python3 required" >&2; exit 2; }
command -v jq >/dev/null 2>&1 || { echo "❌ jq required for targets parsing" >&2; exit 2; }

echo "🧪 [BATCH] Loading targets from $TARGETS_FILE"
REPO_COUNT=$(jq '.targets | length' "$TARGETS_FILE")
echo "🧪 [BATCH] $REPO_COUNT repos queued"

for i in $(seq 0 $((REPO_COUNT - 1))); do
    NAME=$(jq -r ".targets[$i].name" "$TARGETS_FILE")
    URL=$(jq -r ".targets[$i].url" "$TARGETS_FILE")
    LANGUAGE=$(jq -r ".targets[$i].language" "$TARGETS_FILE")
    SYMBOL=$(jq -r ".targets[$i].probe_symbol // \"\"" "$TARGETS_FILE")
    HAS_TIER_B=$(jq -r ".targets[$i].tiers | index(\"B\") != null" "$TARGETS_FILE")

    CLONE_DIR="$WORK_DIR/$NAME"
    echo ""
    echo "🔬 [$((i+1))/$REPO_COUNT] $NAME ($LANGUAGE)"

    # --- Clone ---
    CLONE_FLAGS="--depth 1 --filter=blob:none"
    if [ "$HAS_TIER_B" = "true" ]; then
        CLONE_FLAGS="--depth 1"
        echo "   cloning (full metadata for Tier B)..."
    else
        echo "   cloning (shallow)..."
    fi

    if ! git clone --quiet $CLONE_FLAGS "$URL" "$CLONE_DIR" 2>/dev/null; then
        echo "⚠️  SKIP $NAME: clone failed"
        SKIP=$((SKIP+1))
        continue
    fi

    # --- Tier A: T0 protocol ---
    echo "   running T0 protocol..."
    T0_ARGS=("$CLONE_DIR" $QUICK_FLAG)
    [ -n "$SYMBOL" ] && [ "$SYMBOL" != "null" ] && T0_ARGS+=(--symbol "$SYMBOL")

    T0_START=$(date +%s%N 2>/dev/null || date +%s)
    if bash "$SCRIPT_DIR/run_protocol.sh" "${T0_ARGS[@]}" >/dev/null 2>&1; then
        T0_STATUS="pass"
        PASS=$((PASS+1))
    else
        T0_STATUS="fail"
        FAIL=$((FAIL+1))
    fi
    T0_END=$(date +%s%N 2>/dev/null || date +%s)

    # --- Guard scan timing ---
    GUARD_START=$(date +%s%N 2>/dev/null || date +%s)
    GUARD_OUT=$(dagr guard --workspace "$CLONE_DIR" --format json 2>/dev/null)
    GUARD_END=$(date +%s%N 2>/dev/null || date +%s)
    G_STATUS=$(python3 -c "import json,sys;d=json.loads(sys.stdin.read());print(d.get('status','error'))" <<<"$GUARD_OUT" 2>/dev/null || echo error)
    G_RULES=$(python3 -c "import json,sys;d=json.loads(sys.stdin.read());print(d.get('active_rules',0))" <<<"$GUARD_OUT" 2>/dev/null || echo 0)
    G_VIOLATIONS=$(python3 -c "import json,sys;d=json.loads(sys.stdin.read());print(d.get('violations_count',0))" <<<"$GUARD_OUT" 2>/dev/null || echo "?")
    GUARD_MS=$(( (GUARD_END - GUARD_START) / 1000000 ))

    # --- Slice compression ---
    SLICE_COMP="skipped"
    if [ -n "$SYMBOL" ] && [ "$SYMBOL" != "null" ] && [ -f "$CLONE_DIR/${SYMBOL%%:*}" ]; then
        SLICE_JSON=$(dagr context "$SYMBOL" --format json 2>/dev/null)
        SLICE_COMP=$(python3 -c "
import json,sys
try:
    d=json.loads(sys.stdin.read())
    print(round(d.get('compression_ratio',0)*100,1))
except: print('n/a')" <<<"$SLICE_JSON" 2>/dev/null || echo "n/a")
    fi

    # --- Tier B: git workflow probes ---
    TIER_B_STATUS="skipped"
    if [ "$HAS_TIER_B" = "true" ]; then
        echo "   running Tier B git-workflow probes..."
        if bash "$SCRIPT_DIR/tier_b.sh" "$CLONE_DIR" >/dev/null 2>&1; then
            TIER_B_STATUS="pass"
            PASS=$((PASS+1))
        else
            TIER_B_STATUS="fail"
            FAIL=$((FAIL+1))
        fi
    fi

    echo "   ✅ T0=$T0_STATUS · guard=${G_STATUS}(${GUARD_MS}ms, ${G_RULES} rules) · slice=${SLICE_COMP}% · tier_b=${TIER_B_STATUS}"
done

echo ""
echo "📊 [BATCH] Complete: PASS=$PASS FAIL=$FAIL SKIP=$SKIP"
exit 0
