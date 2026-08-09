# Tandem Game Companion Architecture

[Documentation index](index.md) · [Guardian and Worker](GUARDIAN_WORKER.md) · [Security Model](SECURITY_MODEL.md) · [Testing](TESTING.md)

---

Tandem is one Rust binary with two runtime roles: **guardian** and **worker**.

## Runtime structure

```text
GameNative, Winlator, Wine, or Windows
└── TandemGameCompanion.exe              guardian
    └── TandemGameCompanion.exe --worker worker
        ├── game
        ├── tools
        └── cmd.exe /D /S /C call Script.cmd ...
```

The guardian owns process-lifetime supervision. The worker owns configuration validation, launch
sequencing, before-game waits, logging, game waiting, and direct-child cleanup.

The worker reports exactly one game PID through a reserved status record. Launched child output is
redirected to the session log instead of the status channel. On Windows, handle inheritance is
cleared for that channel before games or tools are started.

## Source modules

| Module | Responsibility |
|---|---|
| `main.rs` | Program entry point and top-level exit handling |
| `cli.rs` | Public CLI parsing and runtime-mode selection |
| `config.rs` | TOML parsing, path validation, file-type checks, and limits |
| `guardian.rs` | Worker supervision, protocol handling, and fallback game wait |
| `launcher.rs` | Launch order, preparation policy, waits, process creation, logging, exit propagation, and cleanup |
| `platform.rs` | Windows process handles, shared PID-scoped window/control discovery, bounded allowlisted Win32 control mutations, status-handle protection, and native confirmation UI |
| `preparation.rs` | Sequential bounded tool-preparation execution and process-exit detection |
| `protocol.rs` | Reserved worker-to-guardian game-PID record |
| `error.rs` | Error types and process exit-code mapping |

## Responsibility boundary

### Guardian

- starts the worker;
- receives and validates the game PID record;
- opens a synchronization-only game handle on Windows;
- forwards ordinary worker output;
- reaps the worker; and
- remains alive until the game exits when the worker fails after game creation.

### Worker

- loads and validates `Tandem.toml`;
- launches before-game tools;
- executes sequential bounded window/control preparation against the directly launched process, including allowlisted ComboBox selection, push-button invocation, auto-checkbox state setting, and standard single-line Edit text setting;
- performs user-confirmation or tool-exit waits;
- starts and reports the game;
- launches after-game tools;
- waits for the game; and
- applies configured direct-child cleanup.

## Current recovery boundary

The recovery model is deliberately limited. Tandem does not yet provide:

- worker restart;
- state reconstruction after a worker crash;
- Windows Job Object or process-group cleanup;
- descendant process-tree tracking; or
- recovery after guardian termination.

See [Guardian and Worker](GUARDIAN_WORKER.md) for the detailed supervision behavior.

## Win32 preparation boundary

The worker delegates standard HWND discovery and allowlisted mutation to `platform.rs`. Mutating
actions are limited to standard ComboBox index selection, push/default-push button invocation,
`BS_AUTOCHECKBOX` checked-state transitions, and standard single-line editable `Edit` text setting.
Edit text setting reads the current text, skips `WM_SETTEXT` when already correct, otherwise sends one
bounded `WM_SETTEXT`, and then requires an exact read-back result. Text contents are not included in
preparation descriptions or result logs.
