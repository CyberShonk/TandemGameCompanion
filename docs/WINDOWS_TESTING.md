# Windows Build and Smoke Testing

[Documentation index](index.md) · [Testing](TESTING.md) · [Troubleshooting](troubleshooting.md) · [Contributing](../CONTRIBUTING.md)

---

This workflow builds the Windows target from Linux and runs the Windows test path through Wine.

## Prerequisites

- Rust
- `cargo-xwin`
- Clang and LLD
- Wine
- MinGW-w64 GCC
- `file`
- `sha256sum`

Install `cargo-xwin`:

```bash
cargo install --locked cargo-xwin
```

## Build the Windows executable

```bash
./scripts/build-windows.sh
```

Expected outputs:

```text
target/windows-release/TandemGameCompanion.exe
target/windows-release/TandemGameCompanion.exe.sha256
```

The checksum record contains only `TandemGameCompanion.exe`, so it remains portable between
machines and directories.

## Run the Wine smoke test

```bash
./scripts/test-windows.sh
```

A successful run covers:

- Windows-target Rust tests;
- release compilation;
- EXE, BAT, and CMD launch paths;
- BAT/CMD argument preservation;
- PID-scoped `wait-for-window` preparation with a competing same-title window;
- process- and parent-window-scoped `wait-for-control` preparation;
- rejection of matching controls in another process or another top-level window;
- rejection of hidden, disabled, and partial ID/class matches;
- sequential window and control preparation;
- control preparation tool-exit detection plus required and optional timeout paths;
- before-game `tool-exit` waiting;
- after-game delays;
- launch ordering;
- exit-status logging; and
- guardian recovery after a simulated worker failure.

The command should exit with status `0`.

## Manual Windows and GameNative checks

Automated Wine coverage does not replace real-environment testing. Manually verify:

- normal game launch;
- a successful and timed-out `wait-for-window` preparation;
- successful and timed-out `wait-for-control` preparation against a real standard Win32 control;
- exact PID ownership, parent-window scoping, hidden/disabled filtering, and ID/class AND semantics;
- the native user-confirmation dialog;
- touch and controller focus mapping;
- Cancel cleanup;
- a setup utility returning zero and nonzero;
- required-tool failure;
- game launch failure after a tool starts;
- a persistent tool with `close_when_game_exits = false`;
- game exit during a delayed launch;
- a guardian-protocol spoof attempt from child output;
- fullscreen window ordering; and
- native-rendering or direct-scanout behavior that obscures secondary windows.

Record the environment, device, game, tool, configuration, and relevant `Tandem.log` for every
compatibility result.
