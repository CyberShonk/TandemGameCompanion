#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(git rev-parse --show-toplevel)"
TARGET="x86_64-pc-windows-msvc"
OUTPUT_DIR="$ROOT_DIR/target/windows-release"
SOURCE_EXE="$ROOT_DIR/target/$TARGET/release/tandem-game-companion.exe"
OUTPUT_EXE="$OUTPUT_DIR/TandemGameCompanion.exe"

cd "$ROOT_DIR"

for command in cargo file sha256sum; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Required command not found: $command" >&2
    exit 1
  fi
done

if ! cargo xwin --version >/dev/null 2>&1; then
  echo "cargo-xwin is not installed. Run: cargo install --locked cargo-xwin" >&2
  exit 1
fi

echo "== Building Tandem for Windows =="
cargo xwin build \
  --release \
  --target "$TARGET"

mkdir -p "$OUTPUT_DIR"

install -m 0755 \
  "$SOURCE_EXE" \
  "$OUTPUT_EXE"

echo
echo "== Windows executable =="
file "$OUTPUT_EXE"

sha256sum "$OUTPUT_EXE" |
  tee "$OUTPUT_EXE.sha256"

echo
echo "Windows build created:"
echo "  $OUTPUT_EXE"
echo "  $OUTPUT_EXE.sha256"
