# Tandem Game Companion User Guide

[Documentation index](index.md) · [Configuration](CONFIGURATION.md) · [Troubleshooting](troubleshooting.md) · [Project README](../README.md)

---

This guide covers the normal path from extracting Tandem to launching a game with one or more
companion tools.

> [!WARNING]
> Tandem is alpha software. Keep backups of important configuration files and test new setups with
> tools and games you trust.

## Basic use path

1. Extract Tandem into the game folder.
2. Put companion programs in a `Tools` folder.
3. Edit `Tandem.toml`.
4. Validate the configuration when possible.
5. Point the Windows or compatibility environment at `TandemGameCompanion.exe`.
6. Launch normally.
7. Check `Tandem.log` if anything fails.

## 1. Prepare the folder

A simple portable layout is easiest to review and move:

```text
GameFolder/
├── TandemGameCompanion.exe
├── Tandem.toml
├── ExampleGame.exe
└── Tools/
    ├── Trainer.exe
    ├── Setup.exe
    └── ControllerUtility.exe
```

Do not rename or replace the original game executable. Tandem should launch the real game file
listed in the configuration.

## 2. Configure the game

Start with [`Tandem.example.toml`](../Tandem.example.toml). The minimum game section looks like:

```toml
config_version = 1

[launcher]
log_file = "Tandem.log"
allow_external_paths = false
continue_on_optional_tool_failure = true

[game]
name = "Example Game"
path = "ExampleGame.exe"
arguments = []
working_directory = "."
```

Use forward slashes in portable paths, including on Windows:

```toml
path = "Tools/Trainer.exe"
```

Keep `allow_external_paths = false` unless the game or tool genuinely must live outside the
folder containing `Tandem.toml`.

## 3. Add tools

Each tool uses its own `[[tools]]` section.

### Normal companion after game startup

```toml
[[tools]]
name = "Controller Utility"
path = "Tools/ControllerUtility.exe"
arguments = []
working_directory = "Tools"
launch = "after-game"
delay_ms = 2000
required = false
close_when_game_exits = true
```

This starts the tool two seconds after the game process starts.

### Trainer configuration before game startup

```toml
[[tools]]
name = "Trainer"
path = "Tools/Trainer.exe"
arguments = []
working_directory = "Tools"
launch = "before-game"
before_game_wait = "user-confirmation"
delay_ms = 0
required = true
close_when_game_exits = true
```

Tandem opens the trainer and shows a native Windows confirmation dialog. The game remains stopped
until the user selects **OK**.

Selecting **Cancel** fails the session and closes tools already started by Tandem. A trainer that
remains open after confirmation follows `close_when_game_exits` when the game ends.

The workflow is intentionally completed before the game starts. Fullscreen, native-rendering, or
direct-scanout modes may cover secondary Windows windows after launch.

### One-shot setup utility

```toml
[[tools]]
name = "Setup Utility"
path = "Tools/Setup.exe"
arguments = []
working_directory = "Tools"
launch = "before-game"
before_game_wait = "tool-exit"
delay_ms = 0
required = true
close_when_game_exits = false
```

Tandem waits for the utility to exit. Exit code `0` continues to the game. A required nonzero exit
stops the session and is returned by Tandem.

## 4. Choose failure behavior

```toml
required = false
```

An optional tool may fail without blocking the game when
`continue_on_optional_tool_failure = true`.

```toml
required = true
```

A required tool must start successfully. For `tool-exit` waiting, it must also exit successfully.
Use this when the game should not start without the tool.

## 5. Choose cleanup behavior

```toml
close_when_game_exits = true
```

Tandem attempts to terminate the direct child process it started when the game exits normally.

```toml
close_when_game_exits = false
```

The tool may remain running after a successful game session. Tools started during a session failure
are still cleaned up where Tandem retains control.

Tandem currently manages the directly launched process, not every descendant that a launcher,
script, or tool may create.

## 6. Validate before launching

From Windows or a Wine command prompt:

```text
TandemGameCompanion.exe --validate
```

Preview the resolved launch plan without starting programs:

```text
TandemGameCompanion.exe --dry-run
```

Validation catches common path, file-type, configuration-version, and script-safety problems.

## 7. Configure GameNative or Winlator

Set the container's main executable to:

```text
TandemGameCompanion.exe
```

Use the folder containing `TandemGameCompanion.exe` and `Tandem.toml` as the working directory.
Do not point the container directly at the game executable when testing Tandem.

The native confirmation dialog should be usable before the game starts. Touch behavior is normally
provided by the compatibility environment. Controller use depends on its controller-to-pointer or
keyboard mapping.

## 8. Read the log

Tandem writes `Tandem.log` beside the configuration by default. The log records:

- the resolved configuration path;
- tool and game launch attempts;
- process IDs;
- delays and wait states;
- exit statuses;
- cleanup results; and
- the final session result.

When reporting a problem, remove personal paths if needed but keep the surrounding launch and exit
messages intact.

## Current limitations

- Configuration is edited manually.
- A visible console window remains during normal launch.
- There is no graphical setup editor or general controller-driven setup interface.
- Cleanup is direct-child based rather than process-tree based.
- Worker recovery does not restart the worker or reconstruct tool cleanup.
- Real-device GameNative and Winlator coverage remains limited.

See [Troubleshooting](troubleshooting.md) for common failures and the
[Configuration Reference](CONFIGURATION.md) for every supported field.
