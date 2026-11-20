#!/bin/bash
# Check out the Taixin vendor firmware tree at the pinned commit and apply patches/*.patch.
# Usage: scripts/apply.sh [target-dir]   (default: TXW8301_WNB next to patches/, ignored by git)
set -e
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
. "$ROOT/scripts/upstream.env"
DIR="${1:-$ROOT/TXW8301_WNB}"

if [ -e "$DIR" ]; then
    echo "$DIR already exists; remove it or pass another directory." >&2
    exit 1
fi

git clone "$FW_UPSTREAM" "$DIR"
cd "$DIR"
git checkout -q -b bridge "$FW_BASE"

# git am needs a committer identity; fall back to a generic one if none is configured.
if [ -z "$(git config user.email)" ]; then
    export GIT_COMMITTER_NAME="${GIT_COMMITTER_NAME:-taixin-halow-bridge}"
    export GIT_COMMITTER_EMAIL="${GIT_COMMITTER_EMAIL:-apply@localhost}"
fi

# --keep-cr: project/txw4002a.cdkproj is stored with CRLF line endings upstream.
git am -q --keep-cr --whitespace=nowarn "$ROOT"/patches/*.patch

echo
echo "Patched tree: $DIR (branch 'bridge', $(git rev-list --count "$FW_BASE"..HEAD) commits on $FW_BASE)"
echo "Open $DIR/project/txw4002a.cdkproj in CDK and build."
