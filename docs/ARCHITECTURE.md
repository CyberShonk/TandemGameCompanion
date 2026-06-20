# Architecture

Tandem is currently a single Rust binary with separate runtime roles and focused source
modules.

## Implemented process model

A normal launch starts two instances of the same executable:

```text
GameNative, Wine, or Windows
└── TandemGameCompanion.exe              Guardian
    └── TandemGameCompanion.exe --worker Worker
        ├── Game.exe
        ├── CompanionTool.exe
        └── cmd.exe /c Script.bat
```

### Guardian

The guardian is the process configured as the main executable. It:

- Starts the worker.
- Reads the game process ID reported by the worker.
- Opens a synchronization handle to that process on Windows.
- Forwards normal worker output.
- Exits normally when the worker completes successfully.
- Remains active until the game exits if the worker fails after reporting the game process.

The guardian does not parse the main configuration or launch companion tools directly.

### Worker

The worker:

- Loads and validates the TOML configuration.
- Launches before-game tools.
- Launches the game and reports its process ID.
- Launches after-game tools.
- Waits for the game to exit.
- Performs configured direct-child cleanup.
- Writes the session log.

## Current source modules

| Module | Responsibility |
|---|---|
| `cli.rs` | Command-line parsing and mode selection |
| `config.rs` | TOML parsing, path resolution, limits, and validation |
| `guardian.rs` | Guardian process lifecycle and worker supervision |
| `launcher.rs` | Launch sequencing, logging, waiting, and cleanup |
| `platform.rs` | Platform-specific process waiting |
| `protocol.rs` | Internal worker-to-guardian game PID message |
| `error.rs` | Application error types and exit-code mapping |

## Current recovery boundary

The guardian preserves the outer process lifetime if the worker exits after game startup.
It does not yet restart the worker, recover companion-tool cleanup, or survive termination of
the guardian itself.

The current game-PID protocol shares the worker's standard-output stream. A dedicated private
IPC channel is planned before the architecture is considered stable.

## Planned roles

The following roles are requirements, not implemented features:

- Graphical configuration interface
- Controller navigation
- Non-activating notifications
- Worker restart and recovery reattachment

## Unsafe code

Windows process synchronization uses small, isolated `unsafe` blocks around `OpenProcess`,
`WaitForSingleObject`, and `CloseHandle`. Each block documents its handle-lifetime and access
assumptions.
