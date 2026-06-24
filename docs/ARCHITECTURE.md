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
| `launcher.rs` | Launch order, waits, process creation, logging, exit propagation, and cleanup |
| `platform.rs` | Windows process handles, status-handle protection, and native confirmation UI |
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
