#!/usr/bin/env bash
# ====================================================================
# T0 Field-Test Harness (skeleton) — TESTING_ROADMAP Phase T0
# Usage: run_protocol.sh <repo-path> [--quick] [--symbol FILE:SYMBOL]
#
# Phases implemented:
#   P0 env snapshot   : dagr version, OS, HEAD SHA, launch CWD (P3 principle)
#   P1 baseline guard : JSON status + active_rules (P2: assert internals)
#   P2 slice probe    : one symbol slice, net-compression capture
#   P3 MCP parity-lite: initialize + tools/list over stdio
#   P4 stats delta / sandbox drills : NOT yet wired (skeleton)
# Output: JSON results file (path echoed).
# ====================================================================
set -u

REPO="${1:?usage: run_protocol.sh <repo-path> [--quick] [--symbol FILE:SYMBOL]}"
shift || true
SYMBOL="tests/fixtures/billing_service.ts:chargeCustomer"
while [ $# -gt 0 ]; do
    case "$1" in
        --quick) shift ;;
        --symbol) shift; SYMBOL="$1" ;;
        *) shift ;;
    esac
done

command -v dagr >/dev/null 2>&1 || { echo "❌ dagr not on PATH" >&2; exit 2; }
REPO=$(cd "$REPO" && pwd)
cd "$REPO" || exit 1

OUT="${DAGR_FIELDTEST_OUT:-$PWD/target/fieldtest-results-$(date +%s).json}"
mkdir -p "$(dirname "$OUT")"

echo "🔬 [T0] field protocol on $REPO"
DV=$(dagr --version 2>/dev/null | head -1)
SHA=$(git rev-parse HEAD 2>/dev/null || echo "no-git")

# --- P1: baseline guard -------------------------------------------------
GUARD=$(dagr guard --workspace "$REPO" --format json 2>/dev/null)
G_STATUS=$(python3 -c "import json,sys;d=json.loads(sys.stdin.read());print(d.get('status','error'))" <<<"$GUARD" 2>/dev/null || echo error)
G_RULES=$(python3 -c "import json,sys;d=json.loads(sys.stdin.read());print(d.get('active_rules',0))" <<<"$GUARD" 2>/dev/null || echo 0)

# --- P2: slice probe ----------------------------------------------------
SLICE_FILE="${SYMBOL%%:*}"
if [ -f "$REPO/$SLICE_FILE" ]; then
    SLICE=$(dagr context "$SYMBOL" --format json 2>/dev/null)
    COMP=$(python3 -c "import json,sys;d=json.loads(sys.stdin.read());print(round(d.get('compression_ratio',0)*100,1))" <<<"$SLICE" 2>/dev/null || echo n/a)
else
    COMP="skipped"
fi

# --- P3: MCP parity-lite (initialize + tools/list) -----------------------
# Core MCP surface = 4 governance tools; A2A swarm tools are experimental
# and only present when built with --features a2a.
REQS=$'{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"t0-harness","version":"0"}}}\n{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}\n'
MCP_OUT=$(printf '%s' "$REQS" | dagr mcp start 2>/dev/null)
TOOLS=$(python3 -c "
import json, sys
count = 0
for line in sys.stdin.read().splitlines():
    try:
        r = json.loads(line)
    except Exception:
        continue
    t = r.get('result', {}).get('tools')
    if isinstance(t, list):
        count = len(t)
print(count)" <<<"$MCP_OUT" 2>/dev/null || echo 0)

# --- assemble results -----------------------------------------------------
python3 - "$OUT" "$REPO" "$SHA" "$DV" "$G_STATUS" "$G_RULES" "$COMP" "$TOOLS" <<'PY'
import datetime, json, sys
out, repo, sha, dv, g_status, g_rules, comp, tools = sys.argv[1:9]
def num(x):
    try: return float(x) if "." in str(x) else int(x)
    except Exception: return x
doc = {
    "timestamp_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "repo": repo,
    "head": sha,
    "dagr_version": dv,
    "phases": {
        "guard_status": g_status,
        "active_rules": num(g_rules),
        "slice_compression_pct": num(comp),
        "mcp_tool_count": num(tools),
    },
}
with open(out, "w") as f:
    f.write(json.dumps(doc, indent=2))
print("📄 results:", out)
PY

OK=1
[ "$G_STATUS" = "passed" ] || OK=0
case "$TOOLS" in ''|*[!0-9]*) OK=0 ;; *) [ "$TOOLS" -ge 4 ] || OK=0 ;; esac

[ "$OK" -eq 1 ] && echo "✅ [T0] smoke PASS" || { echo "❌ [T0] smoke FAIL"; exit 1; }
exit 0
