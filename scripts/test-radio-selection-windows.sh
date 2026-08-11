#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(git rev-parse --show-toplevel)"
FIXTURE_DIR="$ROOT_DIR/tests/windows-smoke"
SMOKE_DIR="$ROOT_DIR/target/windows-smoke-radio-selection"
RELEASE_EXE="$ROOT_DIR/target/windows-release/TandemGameCompanion.exe"
SMOKE_EXE="$SMOKE_DIR/TandemGameCompanion.exe"
export WINEPREFIX="${WINEPREFIX:-$HOME/.local/share/tandem-game-companion/wine-prefix}"
export WINEARCH="${WINEARCH:-win64}"
export WINEDEBUG="${WINEDEBUG:--all}"
cd "$ROOT_DIR"
for command in wine wineboot x86_64-w64-mingw32-gcc tr grep; do command -v "$command" >/dev/null 2>&1 || { echo "Required command not found: $command" >&2; exit 1; }; done
[[ -f "$RELEASE_EXE" ]] || "$ROOT_DIR/scripts/build-windows.sh"
mkdir -p "$(dirname "$WINEPREFIX")"; [[ -f "$WINEPREFIX/system.reg" ]] || wineboot -u
rm -rf "$SMOKE_DIR"; mkdir -p "$SMOKE_DIR"; cp "$RELEASE_EXE" "$SMOKE_EXE"
x86_64-w64-mingw32-gcc -O2 -Wall -Wextra -Werror -s -o "$SMOKE_DIR/SmokeGame.exe" "$FIXTURE_DIR/smoke-helper.c"
cp "$SMOKE_DIR/SmokeGame.exe" "$SMOKE_DIR/SmokeTool.exe"
x86_64-w64-mingw32-gcc -O2 -Wall -Wextra -Werror -s -o "$SMOKE_DIR/RadioTool.exe" "$FIXTURE_DIR/radio-helper.c"
for fixture in "$FIXTURE_DIR"/RadioSelection*.toml; do cp "$fixture" "$SMOKE_DIR/"; done
cd "$SMOKE_DIR"
normalize_file(){ tr -d '\r' < "$1" > "$2"; }
assert_contains(){ grep -F -- "$2" "$1" >/dev/null || { echo "Expected text not found in $1: $2" >&2; exit 1; }; }
assert_absent(){ if grep -F -- "$2" "$1" >/dev/null; then echo "Forbidden text found in $1: $2" >&2; exit 1; fi; }
assert_no_violations(){ if [[ -f "$1" ]] && grep -F -- "VIOLATION-" "$1" >/dev/null; then echo "Radio helper reported a mutation-boundary violation:" >&2; grep -F -- "VIOLATION-" "$1" >&2; exit 1; fi; }
run_expected_failure(){ local config="$1" output="$2"; shift 2; rm -f radio-*-events.txt RadioSelection*.log *.raw *.txt; if wine ./TandemGameCompanion.exe --config "$config" > "$output.raw" 2>&1; then echo "Expected $config to fail, but Tandem returned success." >&2; exit 1; fi; normalize_file "$output.raw" "$output"; rm -f "$output.raw"; for expected in "$@"; do assert_contains "$output" "$expected"; done; for events in radio-*-events.txt; do [[ -f "$events" ]] || continue; normalize_file "$events" "$events.normalized"; assert_no_violations "$events.normalized"; assert_absent "$events.normalized" "-game-start"; done; }
echo "== Radio selection success, no-op, group behavior, and isolation =="
rm -f radio-events.txt RadioSelectionMain.log RadioSelectionMain.console.raw RadioSelectionMain.console.txt
wine ./TandemGameCompanion.exe --config RadioSelectionMain.toml --validate
wine ./TandemGameCompanion.exe --config RadioSelectionMain.toml > RadioSelectionMain.console.raw 2>&1
normalize_file RadioSelectionMain.console.raw RadioSelectionMain.console.txt; normalize_file radio-events.txt radio-events.normalized.txt; normalize_file RadioSelectionMain.log RadioSelectionMain.normalized.log; cat radio-events.normalized.txt; assert_no_violations radio-events.normalized.txt
for event in "radio-impostor-start" "radio-impostor-ready-target-0-sibling-1-click-0" "radio-scoped-start" "radio-selector-decoys-ready" "radio-target-hidden-target-0-sibling-1-click-0" "radio-other-window-ready-target-0-sibling-1-click-0" "radio-target-visible-disabled-target-0-sibling-1-click-0" "radio-target-visible-enabled-target-0-sibling-1-click-0" "radio-target-click-1-target-1-sibling-0" "radio-target-final-target-1-sibling-0-click-1" "radio-wrong-id-final-target-0-click-0" "radio-other-window-final-target-0-sibling-1-click-0" "radio-other-process-final-target-0-sibling-1-click-0" "radio-noop-ready-target-1-sibling-0-click-0" "radio-noop-final-target-1-sibling-0-click-0" "radio-main-game-start" "radio-main-game-end"; do assert_contains radio-events.normalized.txt "$event"; done
assert_contains RadioSelectionMain.normalized.log 'selected standard Win32 auto-radio button in window "Scoped Radio Trainer" with selector control ID 5001, runtime class "Button", and button type style 0x0009: prior selected=false, resulting selected=true, sent one bounded BM_CLICK and verified selected state'
assert_contains RadioSelectionMain.normalized.log 'selected standard Win32 auto-radio button in window "Already Selected Radio Trainer" with selector control ID 5002, runtime class "Button", and button type style 0x0009: prior selected=true, resulting selected=true, no click; radio button was already selected'
echo; echo "== Invalid radio-selection recipe validation =="
if wine ./TandemGameCompanion.exe --config RadioSelectionInvalidRecipes.toml --validate > invalid.raw 2>&1; then echo "Expected invalid radio recipe validation to fail." >&2; exit 1; fi
normalize_file invalid.raw invalid.txt
for expected in 'must define exactly one of window_title_equals or window_title_contains' 'must define control_id for deterministic radio-button selection' 'control_id must be between 1 and 2147483647 for select-radio-button' 'control_class_equals must be exactly "Button" for select-radio-button' 'timeout_ms must be between 1 and 120000'; do assert_contains invalid.txt "$expected"; done
echo; echo "== Hidden radio rejection =="; run_expected_failure RadioSelectionHidden.toml hidden.txt 'timed out after 500 ms selecting radio button for companion tool Radio Hidden Tool' 'matching visible enabled descendant control is not available'
echo; echo "== Disabled radio rejection =="; run_expected_failure RadioSelectionDisabled.toml disabled.txt 'timed out after 500 ms selecting radio button for companion tool Radio Disabled Tool' 'matching visible enabled descendant control is not available'
echo; echo "== Manual radio-style rejection =="; run_expected_failure RadioSelectionManual.toml manual.txt 'unsupported button type style 0x0004' 'select-radio-button supports only BS_AUTORADIOBUTTON'
echo; echo "== Checkbox-style rejection =="; run_expected_failure RadioSelectionCheckbox.toml checkbox.txt 'unsupported button type style 0x0003' 'select-radio-button supports only BS_AUTORADIOBUTTON'
echo; echo "== Push-button-style rejection =="; run_expected_failure RadioSelectionPush.toml push.txt 'unsupported button type style 0x0000' 'select-radio-button supports only BS_AUTORADIOBUTTON'
echo; echo "== Owner-draw-style rejection =="; run_expected_failure RadioSelectionOwnerDraw.toml ownerdraw.txt 'unsupported button type style 0x000b' 'select-radio-button supports only BS_AUTORADIOBUTTON'
echo; echo "== Wrong runtime class rejection =="; run_expected_failure RadioSelectionWrongRuntimeClass.toml wrong-runtime.txt 'unsupported runtime class "Static"; expected exactly "Button"'
echo; echo "== Ambiguous parent rejection =="; run_expected_failure RadioSelectionAmbiguousParent.toml ambiguous-parent.txt 'ambiguous parent window selector for companion process' '2 visible top-level windows matched title equals "Ambiguous Radio Parent"'
echo; echo "== Ambiguous control rejection =="; run_expected_failure RadioSelectionAmbiguousControl.toml ambiguous-control.txt 'ambiguous control selector in window "Ambiguous Control Radio Trainer"' '2 visible enabled descendant controls matched control ID 5001 and class equals "Button"'
echo; echo "== Optional radio-selection failure continuation =="
rm -f radio-optionalfailure-events.txt RadioSelectionOptionalFailure.log optional.raw optional.txt
wine ./TandemGameCompanion.exe --config RadioSelectionOptionalFailure.toml > optional.raw 2>&1; normalize_file optional.raw optional.txt; assert_contains optional.txt 'unsupported button type style 0x0004'; normalize_file radio-optionalfailure-events.txt radio-optionalfailure-events.normalized.txt; assert_no_violations radio-optionalfailure-events.normalized.txt; assert_contains radio-optionalfailure-events.normalized.txt 'radio-optional-game-start'; assert_contains radio-optionalfailure-events.normalized.txt 'radio-optional-game-end'
echo; echo "== Radio-selection direct tool-exit detection =="; run_expected_failure RadioSelectionExit.toml exit.txt 'Radio Exit Tool exited before the requested radio button was selected and verified'
echo; printf '%s\n' 'Radio-selection Windows smoke tests passed.'; printf 'Wine prefix: %s\n' "$WINEPREFIX"; printf 'Smoke-test directory: %s\n' "$SMOKE_DIR"
