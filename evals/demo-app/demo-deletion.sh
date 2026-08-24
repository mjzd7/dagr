#!/usr/bin/env sh
# Removes db/client.ts while app.ts still imports it -> review-diff must BLOCK.
set -e
git rm -q src/db/client.ts
git add -A
