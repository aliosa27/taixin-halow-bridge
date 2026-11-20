#!/bin/bash
# Regenerate patches/ from the commits on top of the vendor base in the patched tree.
# Usage: scripts/export-patches.sh [tree-dir]   (default: TXW8301_WNB)
set -e
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
. "$ROOT/scripts/upstream.env"
DIR="${1:-$ROOT/TXW8301_WNB}"

cd "$DIR"
if [ -n "$(git status --porcelain --untracked-files=no)" ]; then
    echo "Uncommitted changes in $DIR; commit them first (CDK-generated files are ignored)." >&2
    git status --short --untracked-files=no >&2
    exit 1
fi

rm -f "$ROOT"/patches/*.patch
git format-patch -q --no-signature -o "$ROOT/patches" "$FW_BASE"..HEAD
echo "$(ls "$ROOT"/patches/*.patch | wc -l) patches written to $ROOT/patches"
