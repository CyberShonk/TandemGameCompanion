# Tandem Game Companion Configuration Reference

[Documentation](index.md) · [User Guide](user-guide.md) · [Troubleshooting](troubleshooting.md)

Tandem reads TOML. The default file is `Tandem.toml` in the current working directory. Use `--config PATH` to select another file.

Start with [`Tandem.example.toml`](../Tandem.example.toml).

## Configuration version

The current schema is:

```toml
config_version = 1
```

Existing version 1 configurations remain compatible when newer optional fields are omitted.

## Basic example

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

[[tools]]
name = "Trainer"
path = "Tools/Trainer.exe"
arguments = []
working_directory = "Tools"
enabled = true
launch = "before-game"
before_game_wait = "user-confirmation"
delay_ms = 0
required = true
close_when_game_exits = true
```

## Main fields

| Section | Field | Default | Meaning |
|---|---|---:|---|
| top level | `config_version` | required | Must be `1` |
| launcher | `log_file` | `Tandem.log` | Session log path |
| launcher | `allow_external_paths` | `false` | Allows paths outside the configuration folder |
| launcher | `continue_on_optional_tool_failure` | `true` | Continues after an optional tool failure |
| game/tool | `name` | required | Label used in output and logs |
| game/tool | `path` | required | EXE, COM, BAT, or CMD path |
| game/tool | `arguments` | `[]` | Argument array |
| game/tool | `working_directory` | program parent | Working directory |
| tool | `enabled` | `true` | Omits the tool when false |
| tool | `launch` | `after-game` | `before-game` or `after-game` |
| tool | `before_game_wait` | `none` | `none`, `user-confirmation`, or `tool-exit` |
| tool | `delay_ms` | `0` | Delay before launch, up to 600,000 ms |
| tool | `required` | `false` | Makes a controlled tool failure fail the session |
| tool | `close_when_game_exits` | `false` | Terminates the direct tool process after normal game exit |

## Paths

Relative paths are resolved from the folder containing the configuration.

With `allow_external_paths = false`, Tandem rejects absolute paths, parent traversal, and resolved paths outside the portable configuration folder.

Program paths must point to files. Working directories must point to directories. Tandem also prevents the session log from overwriting the configuration, game, or configured tool files.

## Supported entry types

Tandem supports:

- `.exe`
- `.com`
- `.bat`
- `.cmd`

BAT and CMD files run through a fixed `cmd.exe` invocation. Arguments must pass Tandem's validation. Shell operators, expansion characters, embedded quotes, control characters, and other unrestricted command text are rejected.

## Tool preparation

Preparation is only valid for a directly launched before game EXE or COM tool.

Steps run in the order they appear. Each step has a bounded timeout from 1 to 120,000 ms. The default is 10,000 ms.

Mutating actions require a numeric control ID. Standard Win32 runtime class and control style are verified before a change is made.

### Common parent selectors

Control actions use exactly one parent window selector:

```toml
window_title_equals = "Trainer"
```

or:

```toml
window_title_contains = "Trainer"
```

Matching is case sensitive.

### `wait-for-window`

```toml
[[tools.prepare]]
action = "wait-for-window"
title_contains = "Trainer"
timeout_ms = 10000
```

Define exactly one of `title_equals` or `title_contains`.

The window must be visible and owned by the exact process Tandem launched.

### `wait-for-control`

```toml
[[tools.prepare]]
action = "wait-for-control"
window_title_contains = "Trainer"
control_class_equals = "ComboBox"
control_id = 1001
timeout_ms = 10000
```

Define `control_id`, `control_class_equals`, or both. When both are present, both must match.

The control must be a visible, enabled descendant of the selected top level window and owned by the same launched process.

### `select-combo-box-index`

```toml
[[tools.prepare]]
action = "select-combo-box-index"
window_title_contains = "Trainer"
control_class_equals = "ComboBox"
control_id = 1001
selected_index = 2
timeout_ms = 10000
```

Fields:

- `control_id` is required and must be from 1 to 65,535.
- `control_class_equals` is optional. When present, it must be `ComboBox`.
- `selected_index` is required, zero based, and must be from 0 to 1,000,000.

The requested item must exist. Tandem verifies the resulting index. If the requested item is already selected, the step succeeds without another change.

### `invoke-button`

```toml
[[tools.prepare]]
action = "invoke-button"
window_title_contains = "Trainer"
control_class_equals = "Button"
control_id = 1002
timeout_ms = 10000
```

Fields:

- `control_id` is required and must be from 1 to 2,147,483,647.
- `control_class_equals` is optional. When present, it must be `Button`.

Only standard `BS_PUSHBUTTON` and `BS_DEFPUSHBUTTON` controls are accepted.

### `set-checkbox-state`

```toml
[[tools.prepare]]
action = "set-checkbox-state"
window_title_contains = "Trainer"
control_class_equals = "Button"
control_id = 1003
checked = true
timeout_ms = 10000
```

Fields:

- `control_id` is required and must be from 1 to 2,147,483,647.
- `control_class_equals` is optional. When present, it must be `Button`.
- `checked` is required and must be `true` or `false`.

Only standard `BS_AUTOCHECKBOX` controls are accepted. An already correct state succeeds without another click. A changed state must verify after the action.

### `select-radio-button`

```toml
[[tools.prepare]]
action = "select-radio-button"
window_title_contains = "Trainer"
control_class_equals = "Button"
control_id = 5002
timeout_ms = 10000
```

Fields:

- `control_id` is required and must be from 1 to 2,147,483,647.
- `control_class_equals` is optional. When present, it must be `Button`.

Only standard `BS_AUTORADIOBUTTON` controls are accepted. An already selected target succeeds without another click. Tandem verifies that a changed target becomes selected. Normal Win32 radio group behavior handles sibling options.

### `set-edit-text`

```toml
[[tools.prepare]]
action = "set-edit-text"
window_title_contains = "Trainer"
control_class_equals = "Edit"
control_id = 4001
text = "60"
timeout_ms = 10000
```

Fields:

- `control_id` is required and must be from 1 to 2,147,483,647.
- `control_class_equals` is optional. When present, it must be `Edit`.
- `text` is required. Empty text is allowed.

Text is limited to 4,096 UTF-16 units and cannot contain NUL, carriage return, or line feed.

Only standard visible, enabled, single line editable `Edit` controls are accepted. Multiline, password, read only, automatic case transforming, OEM transforming, and unsupported custom controls are rejected.

Tandem verifies the resulting text. Preparation descriptions and result logs report text lengths rather than field contents.

## Preparation limits

Preparation deliberately does not provide:

- arbitrary configurable Windows messages;
- keyboard or mouse macros;
- clipboard automation;
- image matching;
- UI Automation;
- custom drawn control mutation; or
- automatic following of a launcher into a different process.

## Before game waits

### Continue immediately

```toml
before_game_wait = "none"
```

### Wait for confirmation

```toml
before_game_wait = "user-confirmation"
```

Tandem displays a native OK/Cancel dialog after the tool's preparation steps finish.

### Wait for tool exit

```toml
before_game_wait = "tool-exit"
```

A required tool must exit with code `0` before the game starts.

## Optional tool failures

```toml
[launcher]
continue_on_optional_tool_failure = true
```

When enabled, an optional tool may fail without blocking the game. A required tool failure still stops the session.

## Cleanup

```toml
close_when_game_exits = true
```

Tandem attempts to terminate the direct tool process it started after normal game exit. Descendant processes created by that tool are outside the current cleanup guarantee.

## Validation commands

Validate without launching:

```text
TandemGameCompanion.exe --validate
```

Print the resolved launch plan without launching:

```text
TandemGameCompanion.exe --dry-run
```
