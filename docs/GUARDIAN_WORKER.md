# Guardian and Worker Process Model

[Documentation index](index.md) · [Architecture](ARCHITECTURE.md) · [Security Model](SECURITY_MODEL.md) · [Testing](TESTING.md)

---

Tandem runs two instances of the same executable during a normal session.

```text
GameNative, Winlator, Wine, or Windows
└── Tandem guardian
    └── Tandem worker
        ├── game
        └── companion tools
```

## Guardian responsibilities

The guardian:

- starts the worker;
- receives one reserved game-PID record;
- rejects duplicate PID records;
- opens a synchronization-only process handle on Windows;
- forwards ordinary worker output;
- reaps the worker before returning protocol or forwarding failures; and
- waits for the game when the worker fails after game startup.

When the worker fails after reporting the game, the guardian returns the worker's meaningful
nonzero exit code after the game exits.

## Worker responsibilities

The worker:

- loads and validates `Tandem.toml`;
- starts before-game tools;
- performs user-confirmation or tool-exit waits;
- starts the game;
- reports the game PID;
- launches after-game tools;
- waits for the game; and
- performs configured direct-child cleanup.

`--worker` is an internal mode and is intentionally omitted from public help output.

## Status-channel isolation

The worker reports the game PID through a reserved status record. Normal child stdout and stderr
are redirected to `Tandem.log`, not to the guardian status stream.

On Windows, the worker also clears handle inheritance on the status handle before creating any
game or tool process. A launched child therefore cannot keep the status channel open or inject a
fake game-PID record through an inherited handle.

## Exit behavior

Tandem preserves meaningful exit codes for:

- the configured game;
- required tools that fail to start;
- required `tool-exit` utilities that return nonzero; and
- worker failures observed by the guardian.

## Current recovery boundary

The guardian preserves container-facing lifetime after worker failure, but it does not:

- restart the worker;
- reconstruct the worker's tracked tool list;
- terminate full descendant process trees;
- reattach cleanup after an unexpected worker crash; or
- recover from guardian termination.
