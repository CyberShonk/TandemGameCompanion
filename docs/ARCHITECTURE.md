# Architecture

Tandem is one Rust binary with guardian and worker runtime roles.

```text
GameNative, Winlator, Wine, or Windows
└── TandemGameCompanion.exe              guardian
    └── TandemGameCompanion.exe --worker worker
        ├── game
        ├── tools
        └── cmd.exe /D /S /C call Script.cmd ...
```

The guardian owns process-lifetime supervision. The worker owns validation, launch sequencing,
before-game waits, logging, game waiting, and direct-child cleanup. The worker reports exactly
one game PID through a reserved stdout record; all launched child output goes to the session
log, and Windows handle inheritance is disabled for the status channel.

| Module | Responsibility |
|---|---|
| `cli.rs` | CLI parsing and mode selection |
| `config.rs` | TOML parsing, canonical path/file validation, and limits |
| `guardian.rs` | Worker supervision, protocol handling, and game fallback wait |
| `launcher.rs` | Sequencing, waits, process creation, logging, exit propagation, cleanup |
| `platform.rs` | Windows process handles/status-handle protection and native confirmation UI |
| `protocol.rs` | Reserved worker-to-guardian PID record |
| `error.rs` | Error types and process exit-code mapping |

The recovery boundary remains direct-process based: no worker restart, job-object/process-group
cleanup, or state reconstruction after a worker crash.
