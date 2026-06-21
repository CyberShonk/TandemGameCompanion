# Guardian and worker process model

```text
GameNative, Winlator, Wine, or Windows
└── Tandem guardian
    └── Tandem worker
        ├── game
        └── companion tools
```

The guardian starts the worker, receives one reserved game-PID record, opens a
synchronization-only process handle on Windows, forwards ordinary worker output, and reaps the
worker. If the worker fails after reporting the game, the guardian waits for the game and then
returns the worker's nonzero exit code.

The worker validates configuration, runs before-game waits, starts and reports the game,
launches after-game tools, waits for the game, and performs direct-child cleanup. Every normal
child stdout/stderr stream is redirected to `Tandem.log`, not to the guardian status stream.
On Windows the worker also clears handle inheritance on that status stream before launching any
child, so a game or tool cannot keep it open or inject guardian records through an inherited
handle.

Protocol and forwarding failures are retained until the worker has been reaped. Duplicate PID
records are rejected. `--worker` remains an internal mode.

## Recovery boundary

The guardian preserves its lifetime when the worker fails after game creation. It does not
restart the worker, reconstruct the worker's tool list, terminate descendant process trees, or
recover from guardian termination.
