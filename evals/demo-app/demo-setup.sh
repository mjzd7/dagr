#!/usr/bin/env sh
# One-time: init git history so review-diff has a base ref.
set -e
[ -d .git ] || { git init -q && git add -A && git -c user.email=demo@dagr -c user.name=demo commit -qm "demo base"; }
echo "demo ready — try: dagr guard --workspace . ; then ./demo-deletion.sh && dagr review-diff HEAD~1 HEAD"
