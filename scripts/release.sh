#!/bin/bash
# Publish a firmware build as the latest release under releases/, which the web UI's online update reads
# (Firmware page -> Check for updates, via raw.githubusercontent.com).
#
# Usage: scripts/release.sh <CDK .bin> "<release notes, one line>"
#
# Steps for a release:
#   1. Bump BRIDGE_RELEASE in TXW8301_WNB/project/project_config.h, commit it, scripts/export-patches.sh.
#   2. Build in CDK (configuration BuildSet) and test the image on a device.
#   3. scripts/release.sh TXW8301_WNB/project/txw8301_v2.4.1.3-43933_app-1_<date>_.bin "What changed"
#   4. Review, commit releases/ and push. Devices see it within ~5 min (GitHub's raw cache).
set -e
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$1"
NOTES="$2"
FW="$ROOT/TXW8301_WNB"
OUT="$ROOT/releases"
MAX=$((0x7C000))                    # OTA size limit (WEBUI_FW_MAX in webui.c)

if [ ! -f "$BIN" ] || [ -z "$NOTES" ]; then
    sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//' >&2
    exit 1
fi

# Version as the firmware reports it (fwver): "v<sdk>-<svn> app-<n> r<n>". SDK and app from makecode's file name.
base="$(basename "$BIN")"
if ! [[ "$base" =~ ^txw8301_(v[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+-[0-9]+)_app-([0-9]+)_ ]]; then
    echo "Expected the makecode image, txw8301_v<sdk>-<svn>_app-<n>_<date>_.bin (not txw8301.bin): $base" >&2
    exit 1
fi
SDKV="${BASH_REMATCH[1]}"
APP="${BASH_REMATCH[2]}"
REL="$(sed -n 's/^#define BRIDGE_RELEASE *\([0-9][0-9]*\).*/\1/p' "$FW/project/project_config.h")"
if [ -z "$REL" ] || [ "$REL" -lt 1 ]; then
    echo "BRIDGE_RELEASE not found in $FW/project/project_config.h" >&2
    exit 1
fi
if [ -n "$(git -C "$FW" status --porcelain --untracked-files=no)" ]; then
    echo "Uncommitted changes in $FW; the image must match a committed tree." >&2
    exit 1
fi
if [ "$BIN" -ot "$FW/project/project_config.h" ]; then
    echo "The image is older than project_config.h; rebuild after bumping BRIDGE_RELEASE." >&2
    exit 1
fi

VERSION="$SDKV app-$APP r$REL"
NAME="txw8301_${SDKV}_app-${APP}_r${REL}.bin"
SIZE=$(stat -c %s "$BIN")
if [ "$SIZE" -gt "$MAX" ]; then
    echo "Image is $SIZE bytes, over the OTA limit of $MAX." >&2
    exit 1
fi
mkdir -p "$OUT"
if [ -f "$OUT/$NAME" ]; then
    echo "$NAME is already released; bump BRIDGE_RELEASE, rebuild and try again." >&2
    exit 1
fi

cp "$BIN" "$OUT/$NAME"
CRC=$(gzip -c "$OUT/$NAME" | tail -c8 | od -An -tx4 -N4 | tr -d ' \n')   # gzip trailer = CRC-32 of the input
DATE=$(date +%Y-%m-%d)
JNOTES=$(printf '%s' "$NOTES" | sed 's/\\/\\\\/g; s/"/\\"/g')

cat > "$OUT/latest.json" <<EOF
{
  "version": "$VERSION",
  "date": "$DATE",
  "file": "$NAME",
  "size": $SIZE,
  "crc32": "$CRC",
  "notes": "$JNOTES",
  "source": "$(git -C "$FW" rev-parse --short HEAD)"
}
EOF

{
    echo "# Releases"
    echo
    echo "## $VERSION ($DATE)"
    echo
    echo "$NOTES"
    echo
    echo "Image: \`$NAME\`, $SIZE bytes, CRC-32 \`$CRC\`; firmware tree commit \`$(git -C "$FW" rev-parse --short HEAD)\`."
    if [ -f "$OUT/CHANGELOG.md" ]; then
        echo
        tail -n +3 "$OUT/CHANGELOG.md"
    fi
} > "$OUT/CHANGELOG.new"
mv "$OUT/CHANGELOG.new" "$OUT/CHANGELOG.md"

git -C "$ROOT" add "$OUT/$NAME" "$OUT/latest.json" "$OUT/CHANGELOG.md"
echo "Released $VERSION as releases/$NAME ($SIZE bytes, crc32 $CRC)."
echo "Review with 'git diff --cached', then commit and push."
