# Tandem Game Companion User Guide

[Documentation](index.md) · [Configuration](CONFIGURATION.md) · [Troubleshooting](troubleshooting.md)

Tandem starts a game and its configured companion tools as one session. This guide covers the normal setup path.

> [!WARNING]
> Tandem is alpha software. Test new configurations with games and tools you trust.

## Basic setup

1. Extract Tandem into the game folder.
2. Put companion programs in a `Tools` folder.
3. Edit `Tandem.toml`.
4. Validate the configuration when possible.
5. Set the Windows or compatibility environment to launch `TandemGameCompanion.exe`.
6. Launch normally.
7. Check `Tandem.log` if anything fails.

A simple layout is:

```text
GameFolder/
├── TandemGameCompanion.exe
├── Tandem.toml
├── ExampleGame.exe
└── Tools/
    ├── Trainer.exe
    └── ControllerUtility.exe
```

Start with [`Tandem.example.toml`](../Tandem.example.toml).

## Configure the game

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

Relative paths are the simplest portable choice. Forward slashes work in Windows paths used by Tandem.

Keep `allow_external_paths = false` unless a game or tool must be outside the folder containing `Tandem.toml`.

## Add a companion tool

Each tool uses a `[[tools]]` section.

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

This starts the tool two seconds after the game starts.

### Start a tool before the game

```toml
[[tools]]
name = "Trainer"
path = "Tools/Trainer.exe"
launch = "before-game"
before_game_wait = "user-confirmation"
required = true
close_when_game_exits = true
```

`before_game_wait` supports:

- `none`: continue after the tool and its preparation steps are started.
- `user-confirmation`: show an OK/Cancel dialog before game launch.
- `tool-exit`: wait for the tool to finish before game launch.

A required `tool-exit` utility must return exit code `0`.

## Tool preparation recipes

Preparation steps run in order after a directly launched before game EXE or COM tool starts.

The parent window and target control must belong to the exact process Tandem launched. Tandem rejects ambiguous, hidden, disabled, unsupported, or timed out targets.

### Wait for a window

```toml
[[tools.prepare]]
action = "wait-for-window"
title_contains = "Trainer"
timeout_ms = 10000
```

This waits for a visible top level window. Use `title_equals` instead when the full title is stable.

### Wait for a control

```toml
[[tools.prepare]]
action = "wait-for-control"
window_title_contains = "Trainer"
control_class_equals = "ComboBox"
control_id = 1001
timeout_ms = 10000
```

Use `control_id`, `control_class_equals`, or both. When both are present, both must match.

### Select a ComboBox item

```toml
[[tools.prepare]]
action = "select-combo-box-index"
window_title_contains = "Trainer"
control_class_equals = "ComboBox"
control_id = 1001
selected_index = 2
timeout_ms = 10000
```

`selected_index` is zero based. Tandem verifies that the requested item exists and that the resulting selection matches. An already selected index succeeds without another change.

### Invoke a push button

```toml
[[tools.prepare]]
action = "invoke-button"
window_title_contains = "Trainer"
control_class_equals = "Button"
control_id = 1002
timeout_ms = 10000
```

Only standard `BS_PUSHBUTTON` and `BS_DEFPUSHBUTTON` controls are supported.

### Set a checkbox state

```toml
[[tools.prepare]]
action = "set-checkbox-state"
window_title_contains = "Trainer"
control_class_equals = "Button"
control_id = 1003
checked = true
timeout_ms = 10000
```

Only standard `BS_AUTOCHECKBOX` controls are supported. If the requested state is already set, the step succeeds without another click.

### Select a radio button

```toml
[[tools.prepare]]
action = "select-radio-button"
window_title_contains = "Trainer"
control_class_equals = "Button"
control_id = 5002
timeout_ms = 10000
```

Only standard `BS_AUTORADIOBUTTON` controls are supported. An already selected target succeeds without another click. Normal Win32 radio group behavior handles sibling options.

### Set Edit text

```toml
[[tools.prepare]]
action = "set-edit-text"
window_title_contains = "Trainer"
control_class_equals = "Edit"
control_id = 4001
text = "60"
timeout_ms = 10000
```

This action supports visible, enabled, standard single line editable `Edit` controls. Empty text is allowed. Text is limited to 4,096 UTF-16 units and cannot contain NUL, carriage return, or line feed. Tandem does not include the configured or existing field content in preparation result logs.

## Required and optional tools

```toml
required = true
```

Use this when the game should not start if the tool fails.

```toml
required = false
```

An optional tool may fail without blocking the game when `continue_on_optional_tool_failure = true`.

## Close tools with the game

```toml
close_when_game_exits = true
```

Tandem attempts to terminate the direct tool process it started after the game exits.

Tandem does not currently guarantee cleanup of additional processes created by a tool or launcher.

## Validate before launching

From Windows or a Wine command prompt:

```text
TandemGameCompanion.exe --validate
```

Preview the resolved launch plan without starting anything:

```text
TandemGameCompanion.exe --dry-run
```

## GameNative and Winlator

Set the container main executable to `TandemGameCompanion.exe` and use the folder containing `TandemGameCompanion.exe` and `Tandem.toml` as the working directory.

Compatibility differs between devices, container versions, games, and tools. A launcher that exits after starting a different game or tool process may fall outside Tandem's current direct process boundary.

## Read the log

Tandem writes `Tandem.log` beside the configuration by default. It records launch attempts, process IDs, preparation steps, delays, waits, exit results, cleanup results, and the final session result.

When reporting a problem, remove credentials and personal information that are not needed to reproduce it.

## Current limitations

- Configuration is edited manually.
- A console window remains visible during normal launch.
- There is no graphical setup editor or general controller driven setup interface.
- Cleanup follows the direct process Tandem started rather than a complete descendant process tree.
- Real device GameNative and Winlator testing remains limited.

See [Troubleshooting](troubleshooting.md) for common failures and the [Configuration Reference](CONFIGURATION.md) for exact field rules.
