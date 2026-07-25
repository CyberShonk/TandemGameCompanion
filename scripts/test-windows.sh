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

for command in cargo wine wineboot x86_64-w64-mingw32-gcc tr grep cut; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Required command not found: $command" >&2
    exit 1
  fi
done

if ! cargo xwin --version >/dev/null 2>&1; then
  echo "cargo-xwin is not installed. Run: cargo install --locked cargo-xwin" >&2
  exit 1
fi

echo "== Preparing isolated Wine prefix =="
mkdir -p "$(dirname "$WINEPREFIX")"

if [[ ! -f "$WINEPREFIX/system.reg" ]]; then
  wineboot -u
fi

echo
echo "== Windows-target unit tests =="
CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUNNER=wine \
cargo xwin test \
  --target "$TARGET"

echo
"$ROOT_DIR/scripts/build-windows.sh"

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

x86_64-w64-mingw32-gcc \
  -O2 \
  -Wall \
  -Wextra \
  -Werror \
  -s \
  -o "$SMOKE_DIR/WindowTool.exe" \
  "$FIXTURE_DIR/window-helper.c"

x86_64-w64-mingw32-gcc \
  -O2 \
  -Wall \
  -Wextra \
  -Werror \
  -s \
  -o "$SMOKE_DIR/ControlTool.exe" \
  "$FIXTURE_DIR/control-helper.c"

x86_64-w64-mingw32-gcc \
  -O2 \
  -Wall \
  -Wextra \
  -Werror \
  -s \
  -o "$SMOKE_DIR/ComboTool.exe" \
  "$FIXTURE_DIR/combo-helper.c"

cp "$SMOKE_DIR/SmokeHelper.exe" "$SMOKE_DIR/SmokeGame.exe"
cp "$SMOKE_DIR/SmokeHelper.exe" "$SMOKE_DIR/SmokeTool.exe"

cp "$FIXTURE_DIR/BeforeTool.cmd" "$SMOKE_DIR/"
cp "$FIXTURE_DIR/AfterTool.bat" "$SMOKE_DIR/"
cp "$FIXTURE_DIR/Tandem.toml" "$SMOKE_DIR/"
cp "$FIXTURE_DIR/Guardian.toml" "$SMOKE_DIR/"
cp "$FIXTURE_DIR/ControlExit.toml" "$SMOKE_DIR/"
cp "$FIXTURE_DIR/ControlRequiredTimeout.toml" "$SMOKE_DIR/"
cp "$FIXTURE_DIR/ControlOptionalTimeout.toml" "$SMOKE_DIR/"
cp "$FIXTURE_DIR/ComboOutOfRange.toml" "$SMOKE_DIR/"
cp "$FIXTURE_DIR/ComboOptionalFailure.toml" "$SMOKE_DIR/"
cp "$FIXTURE_DIR/ComboExit.toml" "$SMOKE_DIR/"
cp "$FIXTURE_DIR/ComboAmbiguousParent.toml" "$SMOKE_DIR/"
cp "$FIXTURE_DIR/ComboAmbiguousControl.toml" "$SMOKE_DIR/"
cp "$FIXTURE_DIR/ComboWrongRuntimeClass.toml" "$SMOKE_DIR/"
cp "$FIXTURE_DIR/ComboInvalidRecipes.toml" "$SMOKE_DIR/"
cp "$FIXTURE_DIR/ComboInvalidIndexType.toml" "$SMOKE_DIR/"

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
  "impostor-start"
  "impostor-window"
  "scoped-start"
  "scoped-window"
  "control-impostor-start"
  "control-impostor-control-ready"
  "scoped-control-start"
  "scoped-control-selector-decoys-ready"
  "scoped-control-hidden"
  "scoped-control-other-window-ready"
  "scoped-control-visible-disabled"
  "scoped-control-visible-enabled"
  "combo-impostor-start"
  "combo-impostor-ready-index-0"
  "combo-scoped-start"
  "combo-selector-decoys-ready"
  "combo-target-hidden-index-0"
  "combo-other-window-ready-index-0"
  "combo-target-visible-disabled-index-0"
  "combo-target-visible-enabled-index-0"
  "combo-target-index-2"
  "combo-target-notification-index-2"
  "combo-correct-id-wrong-class-unchanged"
  "combo-correct-class-wrong-id-index-0"
  "combo-other-window-index-0"
  "combo-other-process-final-index-0"
  "combo-noop-ready-index-2"
  "combo-noop-final-index-2-no-notification"
  "before-cmd:before-cmd-arg"
  "game-start"
  "after-bat:after-bat-arg"
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

if grep -Fq "VIOLATION-" smoke-events.normalized.txt; then
  echo "Combo-box smoke fixture recorded an unintended mutation or notification." >&2
  grep -F "VIOLATION-" smoke-events.normalized.txt >&2
  exit 1
fi

impostor_window_line="$(grep -nF -m 1 -x "impostor-window" smoke-events.normalized.txt | cut -d: -f1)"
scoped_start_line="$(grep -nF -m 1 -x "scoped-start" smoke-events.normalized.txt | cut -d: -f1)"
scoped_window_line="$(grep -nF -m 1 -x "scoped-window" smoke-events.normalized.txt | cut -d: -f1)"
game_start_line="$(grep -nF -m 1 -x "game-start" smoke-events.normalized.txt | cut -d: -f1)"

if [[ -z "$impostor_window_line" || -z "$scoped_start_line" || "$impostor_window_line" -ge "$scoped_start_line" ]]; then
  echo "The competing same-title window was not ready before the scoped tool started." >&2
  exit 1
fi

if [[ -z "$scoped_window_line" || -z "$game_start_line" || "$scoped_window_line" -ge "$game_start_line" ]]; then
  echo "Game started before the launched tool's own matching window appeared." >&2
  exit 1
fi

control_impostor_ready_line="$(grep -nF -m 1 -x "control-impostor-control-ready" smoke-events.normalized.txt | cut -d: -f1)"
scoped_control_start_line="$(grep -nF -m 1 -x "scoped-control-start" smoke-events.normalized.txt | cut -d: -f1)"
selector_decoys_line="$(grep -nF -m 1 -x "scoped-control-selector-decoys-ready" smoke-events.normalized.txt | cut -d: -f1)"
hidden_control_line="$(grep -nF -m 1 -x "scoped-control-hidden" smoke-events.normalized.txt | cut -d: -f1)"
other_window_control_line="$(grep -nF -m 1 -x "scoped-control-other-window-ready" smoke-events.normalized.txt | cut -d: -f1)"
disabled_control_line="$(grep -nF -m 1 -x "scoped-control-visible-disabled" smoke-events.normalized.txt | cut -d: -f1)"
enabled_control_line="$(grep -nF -m 1 -x "scoped-control-visible-enabled" smoke-events.normalized.txt | cut -d: -f1)"

if [[ -z "$control_impostor_ready_line" || -z "$scoped_control_start_line" || "$control_impostor_ready_line" -ge "$scoped_control_start_line" ]]; then
  echo "The matching control in another process was not ready before the scoped control tool started." >&2
  exit 1
fi

if [[ -z "$selector_decoys_line" || -z "$hidden_control_line" || "$selector_decoys_line" -ge "$hidden_control_line" ]]; then
  echo "The control ID/class AND-semantics decoys were not ready before the target control wait." >&2
  exit 1
fi

if [[ -z "$other_window_control_line" || -z "$disabled_control_line" || "$other_window_control_line" -ge "$disabled_control_line" ]]; then
  echo "The matching control in the other top-level window was not ready before the target control became visible." >&2
  exit 1
fi

if [[ -z "$hidden_control_line" || -z "$disabled_control_line" || "$hidden_control_line" -ge "$disabled_control_line" ]]; then
  echo "The target control did not begin hidden before becoming visible and disabled." >&2
  exit 1
fi

if [[ -z "$disabled_control_line" || -z "$enabled_control_line" || "$disabled_control_line" -ge "$enabled_control_line" ]]; then
  echo "The target control did not remain disabled before becoming enabled." >&2
  exit 1
fi

if [[ -z "$enabled_control_line" || -z "$game_start_line" || "$enabled_control_line" -ge "$game_start_line" ]]; then
  echo "Game started before the correctly scoped control became visible and enabled." >&2
  exit 1
fi

combo_hidden_line="$(grep -nF -m 1 -x "combo-target-hidden-index-0" smoke-events.normalized.txt | cut -d: -f1)"
combo_disabled_line="$(grep -nF -m 1 -x "combo-target-visible-disabled-index-0" smoke-events.normalized.txt | cut -d: -f1)"
combo_enabled_line="$(grep -nF -m 1 -x "combo-target-visible-enabled-index-0" smoke-events.normalized.txt | cut -d: -f1)"
combo_selected_line="$(grep -nF -m 1 -x "combo-target-index-2" smoke-events.normalized.txt | cut -d: -f1)"
combo_notification_line="$(grep -nF -m 1 -x "combo-target-notification-index-2" smoke-events.normalized.txt | cut -d: -f1)"
combo_noop_final_line="$(grep -nF -m 1 -x "combo-noop-final-index-2-no-notification" smoke-events.normalized.txt | cut -d: -f1)"

if [[ -z "$combo_hidden_line" || -z "$combo_disabled_line" || "$combo_hidden_line" -ge "$combo_disabled_line" ]]; then
  echo "The ComboBox target was not proven unchanged while hidden." >&2
  exit 1
fi

if [[ -z "$combo_enabled_line" || -z "$combo_selected_line" || "$combo_enabled_line" -ge "$combo_selected_line" ]]; then
  echo "The ComboBox was selected before it became visible and enabled." >&2
  exit 1
fi

if [[ -z "$combo_selected_line" || -z "$combo_notification_line" || -z "$combo_noop_final_line" || -z "$game_start_line" ]]; then
  echo "ComboBox selection, notification, no-op, or game-start evidence is incomplete." >&2
  exit 1
fi

if [[ "$combo_selected_line" -ge "$game_start_line" || "$combo_notification_line" -ge "$game_start_line" || "$combo_noop_final_line" -ge "$game_start_line" ]]; then
  echo "Game started before ComboBox selection, notification, and no-op verification completed." >&2
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

if ! grep -Fq 'Preparation step 1 for Scoped Window Tool completed: matched window "Shared Trainer Window".' Tandem.log; then
  echo "Scoped Window Tool preparation did not complete according to Tandem.log." >&2
  exit 1
fi

if ! grep -Fq 'Preparation step 1 for Scoped Control Tool completed: matched window "Scoped Trainer Controls".' Tandem.log; then
  echo "Scoped Control Tool wait-for-window preparation did not complete according to Tandem.log." >&2
  exit 1
fi

if ! grep -Fq 'Preparation step 2 for Scoped Control Tool completed: matched visible enabled control ID 1001 with class "ComboBox" in window "Scoped Trainer Controls".' Tandem.log; then
  echo "Scoped Control Tool wait-for-control preparation did not complete according to Tandem.log." >&2
  exit 1
fi

if ! grep -Fq 'Preparation step 3 for Scoped Combo Tool completed: selected standard Win32 ComboBox in window "Scoped Combo Trainer" with selector control ID 1001 and runtime class "ComboBox": requested index 2, prior index 0, resulting index 2, sent one WM_COMMAND/CBN_SELCHANGE notification.' Tandem.log; then
  echo "Scoped Combo Tool selection was not verified according to Tandem.log." >&2
  exit 1
fi

if ! grep -Fq 'Preparation step 1 for Combo No-op Tool completed: selected standard Win32 ComboBox in window "Already Selected Combo Trainer" with selector control ID 1002 and runtime class "ComboBox": requested index 2, prior index 2, resulting index 2, no notification; requested index was already selected.' Tandem.log; then
  echo "Combo No-op Tool did not follow the documented no-op policy." >&2
  exit 1
fi

if ! grep -Fq "Before CMD Tool exited before game launch with status: exit code: 0" Tandem.log; then
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
echo "== Required control tool-exit detection =="
rm -f ControlExit.log control-exit-events.txt control-exit-output.txt

set +e
wine ./TandemGameCompanion.exe --config ControlExit.toml > control-exit-output.txt 2>&1
control_exit_status=$?
set -e

if [[ "$control_exit_status" -ne 1 ]]; then
  echo "Control exit test returned $control_exit_status instead of 1." >&2
  cat control-exit-output.txt >&2
  exit 1
fi

if grep -Fxq "control-exit-game-start" control-exit-events.txt 2>/dev/null; then
  echo "Game started after the required control tool exited before preparation completed." >&2
  exit 1
fi

if ! grep -Fq "exited before a matching visible enabled control appeared with exit code 0" control-exit-output.txt; then
  echo "Control exit test did not report tool-exit detection." >&2
  cat control-exit-output.txt >&2
  exit 1
fi

echo
echo "== Required control timeout failure =="
rm -f ControlRequiredTimeout.log control-required-timeout-events.txt control-required-timeout-output.txt

set +e
wine ./TandemGameCompanion.exe --config ControlRequiredTimeout.toml > control-required-timeout-output.txt 2>&1
required_timeout_status=$?
set -e

if [[ "$required_timeout_status" -ne 1 ]]; then
  echo "Required control timeout test returned $required_timeout_status instead of 1." >&2
  cat control-required-timeout-output.txt >&2
  exit 1
fi

if grep -Fxq "control-required-timeout-game-start" control-required-timeout-events.txt 2>/dev/null; then
  echo "Game started after a required control preparation timeout." >&2
  exit 1
fi

if ! grep -Fq "timed out after 300 ms waiting for companion tool Required Control Timeout Tool visible enabled control" control-required-timeout-output.txt; then
  echo "Required control timeout was not reported deterministically." >&2
  cat control-required-timeout-output.txt >&2
  exit 1
fi

if ! grep -Fq "Closing companion tool Required Control Timeout Tool after preparation failure." ControlRequiredTimeout.log; then
  echo "Required timed-out control tool was not cleaned up." >&2
  exit 1
fi

echo
echo "== Optional control timeout continuation =="
rm -f ControlOptionalTimeout.log control-optional-timeout-events.txt control-optional-timeout-events.normalized.txt control-optional-timeout-output.txt

wine ./TandemGameCompanion.exe --config ControlOptionalTimeout.toml > control-optional-timeout-output.txt 2>&1

if [[ ! -f control-optional-timeout-events.txt ]]; then
  echo "Optional control timeout test did not create its event file." >&2
  cat control-optional-timeout-output.txt >&2
  exit 1
fi

tr -d '\r' < control-optional-timeout-events.txt > control-optional-timeout-events.normalized.txt

if ! grep -Fxq "control-optional-timeout-game-start" control-optional-timeout-events.normalized.txt; then
  echo "Game did not start after an optional control preparation timeout." >&2
  cat control-optional-timeout-output.txt >&2
  exit 1
fi

if ! grep -Fq "Optional tool Optional Control Timeout Tool preparation failed:" ControlOptionalTimeout.log; then
  echo "Optional control timeout was not logged." >&2
  exit 1
fi

if ! grep -Fq "Continuing without this tool." ControlOptionalTimeout.log; then
  echo "Optional control timeout did not continue according to policy." >&2
  exit 1
fi

if ! grep -Fq "Closing companion tool Optional Control Timeout Tool after preparation failure." ControlOptionalTimeout.log; then
  echo "Optional timed-out control tool was not cleaned up." >&2
  exit 1
fi

echo
echo "Control failure-path smoke tests passed."

echo
echo "== Invalid ComboBox recipe validation =="
rm -f combo-invalid-output.txt
set +e
wine ./TandemGameCompanion.exe --config ComboInvalidRecipes.toml --validate > combo-invalid-output.txt 2>&1
combo_invalid_status=$?
set -e
if [[ "$combo_invalid_status" -ne 1 ]]; then
  echo "Invalid ComboBox recipe validation returned $combo_invalid_status instead of 1." >&2
  cat combo-invalid-output.txt >&2
  exit 1
fi
for expected in \
  "must define exactly one of window_title_equals or window_title_contains" \
  "must define control_id for deterministic ComboBox parent notification" \
  "selected_index must be between 0 and 1000000" \
  "must define selected_index" \
  "timeout_ms must be between 1 and 120000" \
  "preparation requires launch = \"before-game\"" \
  "preparation requires a directly launched EXE or COM file"; do
  if ! grep -Fq "$expected" combo-invalid-output.txt; then
    echo "Invalid ComboBox recipe validation did not report: $expected" >&2
    cat combo-invalid-output.txt >&2
    exit 1
  fi
done

echo
echo "== Invalid ComboBox index type validation =="
rm -f combo-invalid-type-output.txt
set +e
wine ./TandemGameCompanion.exe --config ComboInvalidIndexType.toml --validate > combo-invalid-type-output.txt 2>&1
combo_invalid_type_status=$?
set -e
if [[ "$combo_invalid_type_status" -ne 1 ]] \
  || ! grep -Fq "could not parse configuration" combo-invalid-type-output.txt \
  || ! grep -Fq "ComboInvalidIndexType.toml" combo-invalid-type-output.txt \
  || ! grep -Fq 'invalid type: string "two", expected i64' combo-invalid-type-output.txt; then
  echo "Invalid ComboBox selected_index type was not rejected during TOML parsing." >&2
  cat combo-invalid-type-output.txt >&2
  exit 1
fi

echo
echo "== Required out-of-range ComboBox failure =="
rm -f ComboOutOfRange.log combo-out-of-range-events.txt combo-out-of-range-output.txt
set +e
wine ./TandemGameCompanion.exe --config ComboOutOfRange.toml > combo-out-of-range-output.txt 2>&1
combo_range_status=$?
set -e
if [[ "$combo_range_status" -ne 1 ]]; then
  echo "Out-of-range ComboBox test returned $combo_range_status instead of 1." >&2
  cat combo-out-of-range-output.txt >&2
  exit 1
fi
if grep -Fq "VIOLATION-" combo-out-of-range-events.txt 2>/dev/null; then
  echo "Out-of-range ComboBox test mutated or notified the fixture." >&2
  cat combo-out-of-range-events.txt >&2
  exit 1
fi
if ! grep -Fq "combo-out-of-range-still-index-0" combo-out-of-range-events.txt; then
  echo "Out-of-range ComboBox test did not prove the selection remained unchanged." >&2
  exit 1
fi
if grep -Fq "combo-out-of-range-game-start" combo-out-of-range-events.txt; then
  echo "Game started after required out-of-range ComboBox failure." >&2
  exit 1
fi
if ! grep -Fq "requested zero-based index 3 is unavailable; current item count is 3" combo-out-of-range-output.txt; then
  echo "Out-of-range ComboBox failure reason was not deterministic." >&2
  cat combo-out-of-range-output.txt >&2
  exit 1
fi
if ! grep -Fq "Closing companion tool Out-of-range Combo Tool after preparation failure." ComboOutOfRange.log; then
  echo "Required out-of-range ComboBox tool was not cleaned up." >&2
  exit 1
fi

echo
echo "== Optional ComboBox failure continuation =="
rm -f ComboOptionalFailure.log combo-optional-events.txt combo-optional-output.txt
wine ./TandemGameCompanion.exe --config ComboOptionalFailure.toml > combo-optional-output.txt 2>&1
if grep -Fq "VIOLATION-" combo-optional-events.txt 2>/dev/null; then
  echo "Optional ComboBox failure mutated or notified the fixture." >&2
  cat combo-optional-events.txt >&2
  exit 1
fi
if ! grep -Fq "combo-optional-game-start" combo-optional-events.txt; then
  echo "Game did not start after optional ComboBox failure." >&2
  exit 1
fi
if ! grep -Fq "Optional tool Optional Out-of-range Combo Tool preparation failed:" ComboOptionalFailure.log \
  || ! grep -Fq "Continuing without this tool." ComboOptionalFailure.log \
  || ! grep -Fq "Closing companion tool Optional Out-of-range Combo Tool after preparation failure." ComboOptionalFailure.log; then
  echo "Optional ComboBox failure policy or cleanup was not logged." >&2
  exit 1
fi

echo
echo "== ComboBox tool-exit detection =="
rm -f ComboExit.log combo-exit-events.txt combo-exit-output.txt
set +e
wine ./TandemGameCompanion.exe --config ComboExit.toml > combo-exit-output.txt 2>&1
combo_exit_status=$?
set -e
if [[ "$combo_exit_status" -ne 1 ]]; then
  echo "ComboBox exit test returned $combo_exit_status instead of 1." >&2
  cat combo-exit-output.txt >&2
  exit 1
fi
if grep -Fq "combo-exit-game-start" combo-exit-events.txt 2>/dev/null; then
  echo "Game started after the ComboBox tool exited during preparation." >&2
  exit 1
fi
if ! grep -Fq "exited before the requested ComboBox index was selected and verified" combo-exit-output.txt; then
  echo "ComboBox exit test did not report direct-tool exit." >&2
  cat combo-exit-output.txt >&2
  exit 1
fi

echo
echo "== Wrong runtime class rejection =="
rm -f ComboWrongRuntimeClass.log combo-wrong-runtime-class-events.txt combo-wrong-runtime-class-output.txt
set +e
wine ./TandemGameCompanion.exe --config ComboWrongRuntimeClass.toml > combo-wrong-runtime-class-output.txt 2>&1
combo_wrong_class_status=$?
set -e
if [[ "$combo_wrong_class_status" -ne 1 ]] \
  || ! grep -Fq 'has unsupported runtime class "Button"; expected exactly "ComboBox"' combo-wrong-runtime-class-output.txt \
  || grep -Fq "VIOLATION-" combo-wrong-runtime-class-events.txt 2>/dev/null \
  || grep -Fq "combo-wrong-runtime-class-game-start" combo-wrong-runtime-class-events.txt 2>/dev/null; then
  echo "The ID-only selector did not reject the wrong runtime class fail-closed." >&2
  cat combo-wrong-runtime-class-output.txt >&2
  exit 1
fi
if ! grep -Fq "Closing companion tool Wrong Runtime Class Tool after preparation failure." ComboWrongRuntimeClass.log; then
  echo "Wrong-runtime-class tool was not cleaned up." >&2
  exit 1
fi

echo
echo "== Ambiguous ComboBox parent rejection =="
rm -f ComboAmbiguousParent.log combo-ambiguous-parent-events.txt combo-ambiguous-parent-output.txt
set +e
wine ./TandemGameCompanion.exe --config ComboAmbiguousParent.toml > combo-ambiguous-parent-output.txt 2>&1
combo_parent_status=$?
set -e
if [[ "$combo_parent_status" -ne 1 ]] \
  || ! grep -Fq "ambiguous parent window selector" combo-ambiguous-parent-output.txt \
  || grep -Fq "combo-ambiguous-parent-game-start" combo-ambiguous-parent-events.txt 2>/dev/null; then
  echo "Ambiguous ComboBox parent was not rejected fail-closed." >&2
  cat combo-ambiguous-parent-output.txt >&2
  exit 1
fi

echo
echo "== Ambiguous ComboBox control rejection =="
rm -f ComboAmbiguousControl.log combo-ambiguous-control-events.txt combo-ambiguous-control-output.txt
set +e
wine ./TandemGameCompanion.exe --config ComboAmbiguousControl.toml > combo-ambiguous-control-output.txt 2>&1
combo_control_status=$?
set -e
if [[ "$combo_control_status" -ne 1 ]] \
  || ! grep -Fq "ambiguous control selector" combo-ambiguous-control-output.txt \
  || grep -Fq "combo-ambiguous-control-game-start" combo-ambiguous-control-events.txt 2>/dev/null; then
  echo "Ambiguous ComboBox control was not rejected fail-closed." >&2
  cat combo-ambiguous-control-output.txt >&2
  exit 1
fi

echo "ComboBox selection and failure-path smoke tests passed."


echo
echo "== Guardian recovery smoke test =="
rm -f Guardian.log guardian-events.txt guardian-events.normalized.txt

set +e
TANDEM_TEST_WORKER_EXIT_AFTER_GAME_START=1 \
wine ./TandemGameCompanion.exe --config Guardian.toml
guardian_status=$?
set -e

if [[ "$guardian_status" -ne 1 ]]; then
  echo "Guardian recovery test returned $guardian_status instead of 1." >&2
  exit 1
fi

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
