# Guardian and worker process model

Tandem uses two instances of the same executable during a normal launch.

```text
GameNative or Wine
└── Tandem guardian
    └── Tandem worker
        ├── game
        └── companion tools
```

## Guardian

The guardian is the process configured as the main executable in GameNative.

It:

- Starts the worker.
- Receives the game process ID from the worker.
- Opens a synchronization handle to the game process on Windows.
- Forwards normal worker output.
- Normally exits when the worker completes.
- Remains active until the game exits if the worker fails after game startup.

The guardian does not parse the main configuration or launch tools directly.

## Worker

The worker:

- Loads and validates `Tandem.toml`.
- Launches before-game tools.
- Launches the game.
- Reports the game process ID to the guardian.
- Launches after-game tools.
- Waits for the game to exit.
- Performs configured tool cleanup.
- Writes the session log.

`--worker` is an internal command-line mode and is intentionally omitted from
the public help output.

## Current recovery boundary

This milestone protects the GameNative-facing guardian from an unexpected worker
exit after the game process has been created.

It does not yet:

- Restart the failed worker.
- Reattach tool cleanup after a worker failure.
- Recover if the guardian itself is terminated.
- Replace the internal stdout protocol with an inherited anonymous pipe.
