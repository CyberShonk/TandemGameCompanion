#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(git rev-parse --show-toplevel)"
TARGET="x86_64-pc-windows-msvc"
FIXTURE_DIR="$ROOT_DIR/tests/windows-smoke"
SMOKE_DIR="$ROOT_DIR/target/windows-smoke"
RELEASE_EXE="$ROOT_DIR/target/windows-release/TandemGameCompanion.exe"
SMOKE_EXE="$SMOKE_DIR/TandemGameCompanion.exe"

export WINEPREFIX="${WINEPREFIX:-$HOME/.local/share/tandem-game-companion/wine-prefix}"
export WINEARCH="${WINEARCH:-win64}"
export WINEDEBUG="${WINEDEBUG:--all}"

cd "$ROOT_DIR"

for command in cargo wine wineboot x86_64-w64-mingw32-gcc tr grep; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Required command not found: $command" >&2
    exit 1
  fi
done

if ! cargo xwin --version >/dev/null 2>&1; then
  echo "cargo-xwin is not installed. Run: cargo install --locked cargo-xwin" >&2
  exit 1
fi

echo "== Windows-target unit tests =="
CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUNNER=wine \
cargo xwin test \
  --target "$TARGET"

echo
"$ROOT_DIR/scripts/build-windows.sh"

echo
echo "== Preparing isolated Wine prefix =="
mkdir -p "$(dirname "$WINEPREFIX")"

if [[ ! -f "$WINEPREFIX/system.reg" ]]; then
  wineboot -u
fi

echo
echo "== Preparing Windows smoke test =="
rm -rf "$SMOKE_DIR"
mkdir -p "$SMOKE_DIR"

cp "$RELEASE_EXE" "$SMOKE_EXE"

x86_64-w64-mingw32-gcc \
  -O2 \
  -Wall \
  -Wextra \
  -Werror \
  -s \
  -o "$SMOKE_DIR/SmokeHelper.exe" \
  "$FIXTURE_DIR/smoke-helper.c"

cp "$SMOKE_DIR/SmokeHelper.exe" "$SMOKE_DIR/SmokeGame.exe"
cp "$SMOKE_DIR/SmokeHelper.exe" "$SMOKE_DIR/SmokeTool.exe"

cp "$FIXTURE_DIR/BeforeTool.cmd" "$SMOKE_DIR/"
cp "$FIXTURE_DIR/AfterTool.bat" "$SMOKE_DIR/"
cp "$FIXTURE_DIR/Tandem.toml" "$SMOKE_DIR/"
cp "$FIXTURE_DIR/Guardian.toml" "$SMOKE_DIR/"

cd "$SMOKE_DIR"

echo
echo "== Validating Windows configuration =="
wine ./TandemGameCompanion.exe --validate

echo
echo "== Resolved Windows launch plan =="
wine ./TandemGameCompanion.exe --dry-run

rm -f Tandem.log smoke-events.txt smoke-events.normalized.txt

echo
echo "== Running Windows smoke test =="
wine ./TandemGameCompanion.exe

if [[ ! -f smoke-events.txt ]]; then
  echo "Smoke test did not create smoke-events.txt." >&2
  exit 1
fi

tr -d '\r' < smoke-events.txt > smoke-events.normalized.txt

echo
echo "== Recorded events =="
cat smoke-events.normalized.txt

expected_events=(
  "before-cmd"
  "game-start"
  "after-bat"
  "exe-tool"
  "game-end"
)

missing=0

for event in "${expected_events[@]}"; do
  if ! grep -Fxq "$event" smoke-events.normalized.txt; then
    echo "Missing smoke-test event: $event" >&2
    missing=1
  fi
done

if (( missing != 0 )); then
  echo "Windows smoke test failed." >&2
  exit 1
fi

if [[ ! -f Tandem.log ]]; then
  echo "Smoke test did not create Tandem.log." >&2
  exit 1
fi

if ! grep -Fq "Game exited with status: exit code: 0" Tandem.log; then
  echo "Game did not exit successfully according to Tandem.log." >&2
  exit 1
fi

if ! grep -Fq "Before CMD Tool already exited with status: exit code: 0" Tandem.log; then
  echo "Before CMD Tool did not exit successfully according to Tandem.log." >&2
  exit 1
fi

if ! grep -Fq "After BAT Tool already exited with status: exit code: 0" Tandem.log; then
  echo "After BAT Tool did not exit successfully according to Tandem.log." >&2
  exit 1
fi

if ! grep -Fq "Companion EXE Tool already exited with status: exit code: 0" Tandem.log; then
  echo "Companion EXE Tool did not exit successfully according to Tandem.log." >&2
  exit 1
fi


echo
echo "== Guardian recovery smoke test =="
rm -f Guardian.log guardian-events.txt guardian-events.normalized.txt

TANDEM_TEST_WORKER_EXIT_AFTER_GAME_START=1 \
wine ./TandemGameCompanion.exe --config Guardian.toml

if [[ ! -f guardian-events.txt ]]; then
  echo "Guardian recovery test did not create guardian-events.txt." >&2
  exit 1
fi

tr -d '\r' < guardian-events.txt > guardian-events.normalized.txt

if ! grep -Fxq "guardian-game-start" guardian-events.normalized.txt; then
  echo "Guardian recovery game did not start." >&2
  exit 1
fi

if ! grep -Fxq "guardian-game-end" guardian-events.normalized.txt; then
  echo "Guardian exited before the recovery game finished." >&2
  exit 1
fi

echo "Guardian remained active until the game exited after a simulated worker failure."

echo
echo "Windows smoke test passed."
echo "Wine prefix: $WINEPREFIX"
echo "Smoke-test directory: $SMOKE_DIR"
