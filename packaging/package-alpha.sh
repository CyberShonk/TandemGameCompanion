#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(git rev-parse --show-toplevel)"
VERSION="${1:-0.1.0-alpha}"
PACKAGE_NAME="Tandem-Game-Companion-v${VERSION}"
DIST_DIR="$ROOT_DIR/target/dist"
PACKAGE_DIR="$DIST_DIR/$PACKAGE_NAME"
OUTPUT_ZIP="$DIST_DIR/$PACKAGE_NAME.zip"
EXE="$ROOT_DIR/target/windows-release/TandemGameCompanion.exe"

cd "$ROOT_DIR"

if ! command -v zip >/dev/null 2>&1; then
  echo "Required command not found: zip" >&2
  echo "Install it inside tandem-dev with: sudo dnf install -y zip" >&2
  exit 1
fi

if [[ ! -f "$EXE" ]]; then
  echo "Windows executable not found. Building it first..."
  ./scripts/build-windows.sh
fi

rm -rf "$PACKAGE_DIR"
mkdir -p "$PACKAGE_DIR/Tools"

install -m 0755 \
  "$EXE" \
  "$PACKAGE_DIR/TandemGameCompanion.exe"

install -m 0644 \
  "packaging/TESTING-INSTRUCTIONS.md" \
  "$PACKAGE_DIR/TESTING-INSTRUCTIONS.md"

install -m 0644 \
  "packaging/Tandem.toml" \
  "$PACKAGE_DIR/Tandem.toml"

touch "$PACKAGE_DIR/Tools/PLACE-TOOLS-HERE.txt"

(
  cd "$DIST_DIR"
  rm -f "$PACKAGE_NAME.zip" "$PACKAGE_NAME.zip.sha256"
  zip -r "$PACKAGE_NAME.zip" "$PACKAGE_NAME"
  sha256sum "$PACKAGE_NAME.zip" > "$PACKAGE_NAME.zip.sha256"
)

echo
echo "Created:"
echo "  $OUTPUT_ZIP"
echo "  $OUTPUT_ZIP.sha256"
