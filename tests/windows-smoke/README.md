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
- Required tool exit and timeout failures during control preparation.
- Optional control timeout continuation and cleanup.
- Tandem remaining alive until the game exits.
- Successful child-process exit statuses.
- Guardian lifetime and nonzero exit preservation after a simulated worker failure.

Run the complete test from the repository root:

```bash
./scripts/test-windows.sh
```

Generated executables, logs, event files, and Wine artifacts remain under `target/`
or the dedicated Wine prefix and are not committed.
