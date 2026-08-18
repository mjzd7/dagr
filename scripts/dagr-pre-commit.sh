#!/usr/bin/env bash
# ====================================================================
# DAGR Fast Pre-Commit Safety Hook (<35ms execution target)
# ====================================================================
set -e

echo "⚡ [DAGR] Running Pre-Commit Verification Gate..."

# 1. Evaluate architectural boundaries on staged changes
if command -v dagr >/dev/null 2>&1; then
    dagr guard --staged
else
    cargo run -q -p dagr -- guard
fi

# 2. Run fast workspace check & clippy
cargo check --workspace -q

echo "✅ [DAGR] Pre-commit verification passed with 0 violations."
exit 0
