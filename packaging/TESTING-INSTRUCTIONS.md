# Tandem Game Companion Alpha Testing Guide

Tandem is early alpha software. Setup is currently manual: edit `Tandem.toml`, launch Tandem instead
of the game executable, and use `Tandem.log` when reporting a problem.

> [!WARNING]
> Only launch games, trainers, scripts, and utilities you trust. Tandem runs them with the current
> user's permissions.

## What you need

- a Windows x64 game;
- a trainer, controller utility, setup program, or other companion tool;
- a text editor; and
- native Windows, GameNative, Winlator, Wine, or another compatible Windows environment.

## 1. Prepare the game folder

Place the package beside the game executable:

```text
GameFolder/
├── TandemGameCompanion.exe
├── Tandem.toml
├── ExampleGame.exe
└── Tools/
    └── CompanionTool.exe
```

Keeping the files together makes the setup easier to review and move. Do not rename or replace the
original game executable.

## 2. Configure the game

Open `Tandem.toml` and update the game entry:

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

Change `ExampleGame.exe` to the actual executable filename. Use forward slashes for portable paths.

## 3. Add a normal companion tool

```toml
[[tools]]
name = "Companion Tool"
path = "Tools/CompanionTool.exe"
arguments = []
working_directory = "Tools"
launch = "after-game"
delay_ms = 2000
required = false
close_when_game_exits = true
```

This launches the tool two seconds after the game starts.

## 4. Configure a trainer before game startup

Use this when a trainer must be opened and adjusted before the game launches:

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

Expected behavior:

1. Tandem launches the trainer.
2. The game remains stopped.
3. Tandem displays a native Windows OK/Cancel prompt.
4. Select **OK** after configuring the trainer.
5. Tandem launches the game.
6. Selecting **Cancel** prevents game launch and closes tools already started by the session.

The prompt is completed before the game starts. Fullscreen or native-rendering modes may cover the
trainer after launch, so this workflow does not require it to remain visible over the game.

Touch should work through normal compatibility-environment input. Controller use depends on the
environment's controller-to-pointer or keyboard mapping.

## 5. Wait for a setup utility

Use this for a one-shot program that must finish before the game starts:

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

Tandem waits for the utility. Exit code `0` continues to the game. A required nonzero exit stops the
session and is returned by Tandem.

## 6. Choose tool behavior

### Required or optional

```toml
required = false
```

The game may still launch if the tool fails and optional failures are enabled.

```toml
required = true
```

The session fails if the tool cannot start. A waited setup utility must also return exit code `0`.
Use this when the game should not start without the tool.

### Close with the game

```toml
close_when_game_exits = true
```

Tandem attempts to terminate the direct process it started when the game exits.

```toml
close_when_game_exits = false
```

The tool may remain running after a successful session.

### Delay a tool

```toml
delay_ms = 5000
```

The value is milliseconds. Delayed after-game tools are skipped if the game exits before the delay
finishes.

## 7. Supported entry types

Tandem supports:

- `.exe`
- `.com`
- `.bat`
- `.cmd`

BAT and CMD arguments are supported when they pass Tandem's script-safety validation. Tandem rejects
shell operators, expansion characters, embedded quotes, control characters, and other unsafe command
text.

Do not use arbitrary shell-command automation.

## 8. Configure GameNative or Winlator

Set the container's main executable to:

```text
TandemGameCompanion.exe
```

Use the folder containing these files as the working directory:

```text
TandemGameCompanion.exe
Tandem.toml
ExampleGame.exe
```

Do not point the container directly at the game executable while testing Tandem.

## 9. Launch and check the log

A normal session should:

1. read `Tandem.toml`;
2. launch before-game tools;
3. complete any configured wait;
4. launch the game;
5. launch after-game tools;
6. stay alive while the game runs;
7. apply configured cleanup; and
8. write `Tandem.log`.

A visible console window is normal in this alpha.

A successful log includes messages similar to:

```text
Tandem Game Companion
Starting game: Example Game
Game process started with PID 1234
Starting companion tool: Companion Tool
Companion Tool started with PID 1250
Tandem is supervising the game process.
Game exited with status: exit code: 0
Tandem session finished.
```

## 10. Validate the configuration

From Windows or a Wine command prompt:

```text
TandemGameCompanion.exe --validate
```

Preview the resolved launch plan without starting anything:

```text
TandemGameCompanion.exe --dry-run
```

These commands may be less convenient to run through some Android compatibility environments.

## Current limitations

- No graphical configuration editor
- No file-picker interface
- No general controller-driven setup interface
- No notifications or silent background mode
- No automatic configuration generation
- No automatic worker restart after crashes
- Direct-child cleanup rather than full process-tree cleanup
- Limited real-device testing across GameNative and Winlator versions
- Limited support for games that launch through replacement processes

Whenever possible, point Tandem directly at the real game executable instead of a launcher.
Tandem cannot prevent the environment from closing the entire Wine session or container.

## Reporting results

Include:

- device or computer model;
- operating system or Android version;
- GameNative, Winlator, or Wine version;
- game name and executable filename;
- trainer or tool name;
- whether each process launched;
- what happened when the game exited;
- a sanitized `Tandem.toml`;
- `Tandem.log`; and
- exact reproduction steps.

Do not include credentials, copyrighted game files, or proprietary third-party executables.
