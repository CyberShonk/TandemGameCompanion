# Tandem Game Companion Alpha Testing Guide

Tandem is alpha software. Configuration is currently manual.

Only launch games, tools, and scripts you trust. Tandem runs them with the current user's normal permissions.

## Prepare the folder

Keep the package beside the game executable:

```text
GameFolder/
├── TandemGameCompanion.exe
├── Tandem.toml
├── ExampleGame.exe
└── Tools/
    └── CompanionTool.exe
```

Open `Tandem.toml` and replace the example game and tool names with the files you are testing.

## Basic game entry

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

## Add a normal companion tool

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

## Configure a tool before the game

Use a confirmation when you need to configure the tool manually before game launch:

```toml
[[tools]]
name = "Trainer"
path = "Tools/Trainer.exe"
launch = "before-game"
before_game_wait = "user-confirmation"
required = true
close_when_game_exits = true
```

Tandem can also use ordered `[[tools.prepare]]` steps to wait for or configure supported standard Win32 controls. See the public configuration reference for the current action list and exact fields.

## Configure GameNative or Winlator

Set the container main executable to:

```text
TandemGameCompanion.exe
```

Use the folder containing `TandemGameCompanion.exe` and `Tandem.toml` as the working directory.

## Validate the configuration

When command line access is available:

```text
TandemGameCompanion.exe --validate
TandemGameCompanion.exe --dry-run
```

## What to check

A normal test should confirm:

1. Tandem starts.
2. Before game tools start in the configured order.
3. Preparation or confirmation completes as expected.
4. The game starts.
5. After game tools start as configured.
6. Tandem remains active while the configured game process runs.
7. Tools configured to close with the game are cleaned up when possible.
8. `Tandem.log` records the result.

## Current limitations

- No graphical configuration editor.
- A console window remains visible during normal launch.
- Cleanup follows the direct process Tandem started rather than every descendant process.
- Launchers that replace themselves with another process may not fit the current process boundary.
- GameNative and Winlator behavior can differ by device and container version.

## Report a result

Include the Tandem release or commit, device or computer model, operating system or Android version, compatibility environment version, game and tool names, what launched, what failed, exact reproduction steps, a sanitized `Tandem.toml`, and the relevant part of `Tandem.log`.

Remove credentials, account information, unnecessary personal paths, and unrelated log content before sharing. Do not include copyrighted game files or proprietary third party executables.
