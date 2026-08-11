# Windows smoke-test fixtures

These files verify the first Windows launcher path under Wine.

The test covers:

- Windows x86-64 MSVC compilation.
- Windows-target unit tests.
- EXE game launching.
- EXE companion-tool launching.
- BAT and CMD companion-tool launching with validated arguments.
- Paths containing spaces.
- PID-scoped visible-window preparation with a same-title competing process.
- Sequential `wait-for-window` and `wait-for-control` preparation.
- Process and parent-top-level-window control scoping.
- Hidden and disabled control rejection.
- Numeric control ID plus exact class-name AND semantics.
- Standard ComboBox item-count, current-index, set-index, and post-notification verification.
- Exact process, parent-window, visibility, enabled-state, ID, and runtime-class mutation scoping.
- One standard parent selection-change notification and an already-selected no-op with none.
- Exactly-once `BM_CLICK` invocation for standard push and default-push buttons.
- Button PID, parent-window, visibility, enabled-state, control-ID, class, and style scoping.
- Checkbox, radio, owner-drawn, wrong-class, ambiguous, timeout, direct-exit, required, and optional button failures.
- Deterministic `BS_AUTOCHECKBOX` checked/unchecked transitions with bounded `BM_GETCHECK`/`BM_CLICK` verification and already-correct no-op behavior.
- Checkbox PID, parent-window, visibility, enabled-state, control-ID, class, style, ambiguity, timeout, direct-exit, required, and optional failure coverage.
- Deterministic `BS_AUTORADIOBUTTON` selection with bounded `BM_GETCHECK`/`BM_CLICK`, exactly-one-click transition, already-selected no-op behavior, and standard sibling auto-clearing.
- Radio PID, parent-window, visibility, enabled-state, control-ID, class, style, ambiguity, timeout, direct-exit, required, and optional failure coverage.
- Standard single-line Edit text changes, exact read-back, already-correct no-op behavior, Unicode text, empty-string clearing, and normal `EN_UPDATE`/`EN_CHANGE` notifications.
- Edit PID, parent-window, visibility, enabled-state, control-ID, runtime-class, style, ambiguity, redaction, timeout, direct-exit, required, and optional failure coverage.
- Invalid recipe, ambiguous parent/control, out-of-range, direct exit, required, and optional failures.
- Required tool exit and timeout failures during control preparation.
- Optional control timeout continuation and cleanup.
- Tandem remaining alive until the game exits.
- Successful child-process exit statuses.
- Guardian lifetime and nonzero exit preservation after a simulated worker failure.

Run the complete test from the repository root:

```bash
./scripts/test-windows.sh
./scripts/test-edit-text-windows.sh
./scripts/test-radio-selection-windows.sh
```

Generated executables, logs, event files, and Wine artifacts remain under `target/`
or the dedicated Wine prefix and are not committed.
