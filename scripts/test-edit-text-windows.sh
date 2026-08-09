#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(git rev-parse --show-toplevel)"
TARGET="x86_64-pc-windows-msvc"
FIXTURE_DIR="$ROOT_DIR/tests/windows-smoke"
SMOKE_DIR="$ROOT_DIR/target/windows-smoke-edit-text"
RELEASE_EXE="$ROOT_DIR/target/windows-release/TandemGameCompanion.exe"
SMOKE_EXE="$SMOKE_DIR/TandemGameCompanion.exe"

export WINEPREFIX="${WINEPREFIX:-$HOME/.local/share/tandem-game-companion/wine-prefix}"
export WINEARCH="${WINEARCH:-win64}"
export WINEDEBUG="${WINEDEBUG:--all}"

cd "$ROOT_DIR"
for command in wine wineboot x86_64-w64-mingw32-gcc tr grep cut; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Required command not found: $command" >&2
    exit 1
  fi
done

if [[ ! -f "$RELEASE_EXE" ]]; then
  "$ROOT_DIR/scripts/build-windows.sh"
fi

mkdir -p "$(dirname "$WINEPREFIX")"
if [[ ! -f "$WINEPREFIX/system.reg" ]]; then
  wineboot -u
fi

rm -rf "$SMOKE_DIR"
mkdir -p "$SMOKE_DIR"
cp "$RELEASE_EXE" "$SMOKE_EXE"

x86_64-w64-mingw32-gcc \
  -O2 \
  -Wall \
  -Wextra \
  -Werror \
  -s \
  -o "$SMOKE_DIR/SmokeGame.exe" \
  "$FIXTURE_DIR/smoke-helper.c"

x86_64-w64-mingw32-gcc \
  -O2 \
  -Wall \
  -Wextra \
  -Werror \
  -s \
  -o "$SMOKE_DIR/EditTool.exe" \
  "$FIXTURE_DIR/edit-helper.c"

for fixture in "$FIXTURE_DIR"/EditText*.toml; do
  cp "$fixture" "$SMOKE_DIR/"
done

cd "$SMOKE_DIR"

normalize_file() {
  local source="$1"
  local destination="$2"
  tr -d '\r' < "$source" > "$destination"
}

assert_contains() {
  local file="$1"
  local expected="$2"
  if ! grep -F -- "$expected" "$file" >/dev/null; then
    echo "Expected text not found in $file: $expected" >&2
    exit 1
  fi
}

assert_absent() {
  local file="$1"
  local forbidden="$2"
  if grep -F -- "$forbidden" "$file" >/dev/null; then
    echo "Forbidden text found in $file: $forbidden" >&2
    exit 1
  fi
}

assert_no_violations() {
  local file="$1"
  if [[ -f "$file" ]] && grep -F -- "VIOLATION-" "$file" >/dev/null; then
    echo "Edit-text helper reported a mutation-boundary violation:" >&2
    grep -F -- "VIOLATION-" "$file" >&2
    exit 1
  fi
}

run_expected_failure() {
  local config="$1"
  local output="$2"
  shift 2
  rm -f edit-events.txt Tandem.log *.log
  if wine ./TandemGameCompanion.exe --config "$config" > "$output.raw" 2>&1; then
    echo "Expected $config to fail, but Tandem returned success." >&2
    exit 1
  fi
  normalize_file "$output.raw" "$output"
  rm -f "$output.raw"
  for expected in "$@"; do
    assert_contains "$output" "$expected"
  done
  if [[ -f edit-events.txt ]]; then
    normalize_file edit-events.txt edit-events.normalized.txt
    assert_no_violations edit-events.normalized.txt
    assert_absent edit-events.normalized.txt "should-not-start"
  fi
}

echo "== Edit-text main success, no-op, Unicode, clear, redaction, and isolation =="
rm -f edit-events.txt EditTextMain.log EditTextMain.console.raw EditTextMain.console.txt EditTextMain.dry-run.raw EditTextMain.dry-run.txt
wine ./TandemGameCompanion.exe --config EditTextMain.toml --validate
wine ./TandemGameCompanion.exe --config EditTextMain.toml --dry-run > EditTextMain.dry-run.raw 2>&1
normalize_file EditTextMain.dry-run.raw EditTextMain.dry-run.txt
wine ./TandemGameCompanion.exe --config EditTextMain.toml > EditTextMain.console.raw 2>&1
normalize_file EditTextMain.console.raw EditTextMain.console.txt
normalize_file edit-events.txt edit-events.normalized.txt
normalize_file EditTextMain.log EditTextMain.normalized.log

cat edit-events.normalized.txt
assert_no_violations edit-events.normalized.txt

for event in \
  "edit-impostor-start" \
  "edit-impostor-ready-unchanged" \
  "edit-scoped-start" \
  "edit-selector-decoys-ready" \
  "edit-target-hidden-utf16-2-update-0-change-0" \
  "edit-other-window-ready-unchanged" \
  "edit-target-visible-disabled-utf16-2-update-0-change-0" \
  "edit-target-visible-enabled-utf16-2-update-0-change-0" \
  "edit-target-final-utf16-2-update-1-change-1" \
  "edit-correct-class-wrong-id-final-unchanged" \
  "edit-other-window-final-unchanged" \
  "edit-other-process-final-unchanged" \
  "edit-noop-ready-utf16-10-update-0-change-0" \
  "edit-noop-final-utf16-10-update-0-change-0" \
  "edit-unicode-ready-utf16-3-update-0-change-0" \
  "edit-unicode-final-utf16-9-update-1-change-1" \
  "edit-clear-ready-utf16-8-update-0-change-0" \
  "edit-clear-final-utf16-0-update-1-change-1" \
  "edit-redaction-ready-utf16-6-update-0-change-0" \
  "edit-redaction-final-utf16-29-update-1-change-1" \
  "edit-main-game-start" \
  "edit-main-game-start-end"
do
  assert_contains edit-events.normalized.txt "$event"
done

if [[ "$(grep -Fxc 'edit-target-en-update-1' edit-events.normalized.txt)" != "4" ]]; then
  echo "Expected exactly four EN_UPDATE notifications across the four changed Edit controls." >&2
  exit 1
fi
if [[ "$(grep -Fxc 'edit-target-en-change-1' edit-events.normalized.txt)" != "4" ]]; then
  echo "Expected exactly four EN_CHANGE notifications across the four changed Edit controls." >&2
  exit 1
fi

assert_contains EditTextMain.normalized.log 'set standard Win32 Edit text in window "Scoped Edit Trainer" with selector control ID 4001 and runtime class "Edit": requested UTF-16 units 2, prior UTF-16 units 2, resulting UTF-16 units 2, sent one bounded WM_SETTEXT and verified exact text'
assert_contains EditTextMain.normalized.log 'set standard Win32 Edit text in window "Already Correct Edit Trainer" with selector control ID 4002 and runtime class "Edit": requested UTF-16 units 10, prior UTF-16 units 10, resulting UTF-16 units 10, no WM_SETTEXT; requested text was already set'
assert_contains EditTextMain.normalized.log 'set standard Win32 Edit text in window "Unicode Edit Trainer" with selector control ID 4003 and runtime class "Edit": requested UTF-16 units 9, prior UTF-16 units 3, resulting UTF-16 units 9, sent one bounded WM_SETTEXT and verified exact text'
assert_contains EditTextMain.normalized.log 'set standard Win32 Edit text in window "Clear Edit Trainer" with selector control ID 4004 and runtime class "Edit": requested UTF-16 units 0, prior UTF-16 units 8, resulting UTF-16 units 0, sent one bounded WM_SETTEXT and verified exact text'
assert_contains EditTextMain.normalized.log 'set standard Win32 Edit text in window "Redaction Edit Trainer" with selector control ID 4005 and runtime class "Edit": requested UTF-16 units 29, prior UTF-16 units 6, resulting UTF-16 units 29, sent one bounded WM_SETTEXT and verified exact text'

secret='runtime-secret-sentinel-4a913'
assert_absent EditTextMain.dry-run.txt "$secret"
assert_absent EditTextMain.console.txt "$secret"
assert_absent EditTextMain.normalized.log "$secret"

echo
echo "== Invalid edit-text recipe validation =="
if wine ./TandemGameCompanion.exe --config EditTextInvalidRecipes.toml --validate > invalid.raw 2>&1; then
  echo "Expected EditTextInvalidRecipes.toml validation to fail." >&2
  exit 1
fi
normalize_file invalid.raw invalid.txt
rm -f invalid.raw
for expected in \
  'must define exactly one of window_title_equals or window_title_contains' \
  'must define control_id for deterministic edit text preparation' \
  'control_id must be between 1 and 2147483647 for set-edit-text' \
  'control_class_equals must be exactly "Edit" for set-edit-text' \
  'must define text for set-edit-text' \
  'text may not contain CR or LF for single-line set-edit-text' \
  'text exceeds the 4096-UTF-16-unit limit for set-edit-text' \
  'timeout_ms must be between 1 and 120000'
do
  assert_contains invalid.txt "$expected"
done

echo
echo "== Hidden edit rejection =="
run_expected_failure EditTextHidden.toml hidden.txt \
  'timed out after 500 ms setting Edit text for companion tool Edit Hidden Tool' \
  'matching visible enabled descendant control is not available'

echo
echo "== Disabled edit rejection =="
run_expected_failure EditTextDisabled.toml disabled.txt \
  'timed out after 500 ms setting Edit text for companion tool Edit Disabled Tool' \
  'matching visible enabled descendant control is not available'

echo
echo "== Multiline edit-style rejection =="
run_expected_failure EditTextMultiline.toml multiline.txt \
  'unsupported style flags 0x0004' \
  'set-edit-text supports only single-line editable controls without password, read-only, or text-transforming styles'

echo
echo "== Password edit-style rejection =="
run_expected_failure EditTextPassword.toml password.txt \
  'unsupported style flags 0x0020'

echo
echo "== Read-only edit-style rejection =="
run_expected_failure EditTextReadonly.toml readonly.txt \
  'unsupported style flags 0x0800'

echo
echo "== Uppercase-transform edit-style rejection =="
run_expected_failure EditTextUppercase.toml uppercase.txt \
  'unsupported style flags 0x0008'

echo
echo "== Lowercase-transform edit-style rejection =="
run_expected_failure EditTextLowercase.toml lowercase.txt \
  'unsupported style flags 0x0010'

echo
echo "== OEM-transform edit-style rejection =="
run_expected_failure EditTextOemConvert.toml oemconvert.txt \
  'unsupported style flags 0x0400'

echo
echo "== Wrong runtime class rejection =="
run_expected_failure EditTextWrongRuntimeClass.toml wrong-runtime.txt \
  'unsupported runtime class "Static"; expected exactly "Edit"'

echo
echo "== Ambiguous parent rejection =="
run_expected_failure EditTextAmbiguousParent.toml ambiguous-parent.txt \
  'ambiguous parent window selector for companion process' \
  '2 visible top-level windows matched title equals "Ambiguous Edit Trainer"'

echo
echo "== Ambiguous control rejection =="
run_expected_failure EditTextAmbiguousControl.toml ambiguous-control.txt \
  'ambiguous control selector in window "Ambiguous Control Edit Trainer"' \
  '2 visible enabled descendant controls matched control ID 4001 and class equals "Edit"'

echo
echo "== Optional edit-text failure continuation =="
rm -f edit-events.txt EditTextOptionalFailure.log optional.raw optional.txt
wine ./TandemGameCompanion.exe --config EditTextOptionalFailure.toml > optional.raw 2>&1
normalize_file optional.raw optional.txt
rm -f optional.raw
normalize_file edit-events.txt edit-events.normalized.txt
assert_no_violations edit-events.normalized.txt
assert_contains optional.txt 'unsupported style flags 0x0800'
assert_contains edit-events.normalized.txt 'edit-optional-game-start'
assert_contains edit-events.normalized.txt 'edit-optional-game-start-end'

echo
echo "== Edit-text direct tool-exit detection =="
run_expected_failure EditTextExit.toml exit.txt \
  'Edit Exit Tool exited before the requested edit text was set and verified'

echo
printf '%s\n' 'Edit-text Windows smoke tests passed.'
printf 'Wine prefix: %s\n' "$WINEPREFIX"
printf 'Smoke-test directory: %s\n' "$SMOKE_DIR"
