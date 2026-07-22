# Tandem Game Companion Configuration Reference

[Documentation index](index.md) · [User Guide](user-guide.md) · [Troubleshooting](troubleshooting.md) · [Security Model](SECURITY_MODEL.md)

---

Tandem reads a TOML configuration. The default is `Tandem.toml` in the current working directory.
Use `--config PATH` to select another file.

Start with [`Tandem.example.toml`](../Tandem.example.toml).

## Configuration compatibility

The schema remains `config_version = 1`. Configurations created for `v0.1.0-alpha` continue to
work because `before_game_wait` defaults to `none` and `prepare` defaults to an empty recipe
when omitted.

## Complete example

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

[[tools.prepare]]
action = "wait-for-window"
title_contains = "Trainer"
timeout_ms = 10000

[[tools.prepare]]
action = "wait-for-control"
window_title_contains = "Trainer"
control_class_equals = "ComboBox"
control_id = 1001
timeout_ms = 10000
```

## Fields

| Section | Field | Default | Meaning |
|---|---|---:|---|
| top level | `config_version` | required | Must currently be `1` |
| launcher | `log_file` | `Tandem.log` | Session-log output path |
| launcher | `allow_external_paths` | `false` | Allows paths outside the configuration folder |
| launcher | `continue_on_optional_tool_failure` | `true` | Continues after an optional launch or waited-tool failure |
| game/tool | `name` | required | Label used in output and logs |
| game/tool | `path` | required | EXE, COM, BAT, or CMD path |
| game/tool | `arguments` | `[]` | Argument array |
| game/tool | `working_directory` | program parent | Existing child working directory |
| tool | `enabled` | `true` | Omits the tool when false |
| tool | `launch` | `after-game` | `before-game` or `after-game` |
| tool | `before_game_wait` | `none` | `none`, `user-confirmation`, or `tool-exit` |
| tool | `delay_ms` | `0` | Delay before launch, up to 600,000 ms |
| tool | `required` | `false` | Makes launch failure, or a waited nonzero exit, fail the session |
| tool | `close_when_game_exits` | `false` | Terminates the directly launched child after normal game exit |
| tool preparation | `action` | required | `wait-for-window` or `wait-for-control` |
| `wait-for-window` | `title_equals` | unset | Exact case-sensitive top-level window-title match |
| `wait-for-window` | `title_contains` | unset | Case-sensitive top-level window-title substring match |
| `wait-for-control` | `window_title_equals` | unset | Exact case-sensitive parent top-level window title |
| `wait-for-control` | `window_title_contains` | unset | Case-sensitive parent top-level window-title substring |
| `wait-for-control` | `control_id` | unset | Numeric Win32 control ID from 1 to 2,147,483,647 |
| `wait-for-control` | `control_class_equals` | unset | Exact case-sensitive Win32 control class name |
| tool preparation | `timeout_ms` | `10000` | Bounded wait from 1 to 120,000 ms |

## Tool preparation recipes

Preparation steps run sequentially after a before-game tool process starts and before
`before_game_wait` is evaluated. Preparation is only valid with `launch = "before-game"`.

### `wait-for-window`

```toml
[[tools.prepare]]
action = "wait-for-window"
title_contains = "Trainer"
timeout_ms = 10000
```

Each step must define exactly one selector:

- `title_equals` matches the complete title.
- `title_contains` matches a substring.

Matching is case-sensitive and applies only to visible top-level windows owned by the exact PID
Tandem launched. A same-title window owned by another process is ignored. Tandem does not focus,
activate, move, click, send input to, or otherwise mutate the matched window.

### `wait-for-control`

```toml
[[tools.prepare]]
action = "wait-for-control"
window_title_contains = "Trainer"
control_class_equals = "ComboBox"
control_id = 1001
timeout_ms = 10000
```

The parent top-level window must define exactly one of `window_title_equals` or
`window_title_contains`. The descendant control must define `control_id`,
`control_class_equals`, or both. When both control fields are present, Tandem requires both to
match.

Discovery is restricted to visible top-level windows and visible, enabled descendant controls owned
by the exact PID Tandem launched. A matching control in another process or under a different
top-level window cannot satisfy the step. Class matching uses the exact standard Win32 window class
name. Tandem does not read control text, use UI Automation, inspect pixels, focus or activate a
window, send input, or mutate the control.

This action is intended for standard HWND-based Win32 controls. Custom-drawn interfaces and controls
that do not expose a normal descendant window, numeric ID, or stable class name are outside this
action's scope.

A required tool or a globally strict optional-tool policy fails the session when preparation times
out, the tool exits before a match appears, or discovery reports an error. With
`continue_on_optional_tool_failure = true`, Tandem logs the failure, terminates the directly
launched optional tool, skips its remaining preparation and wait behavior, and continues without it.

Preparation requires a directly launched EXE or COM tool. BAT/CMD entries are rejected because the
direct child would be `cmd.exe`. The current process boundary also does not follow child processes,
so a launcher that exits and creates a separate GUI process cannot be matched by either action.

## Before-game wait modes

### `none`

Starts the before-game tool and continues immediately. This preserves the original Tandem behavior.

```toml
launch = "before-game"
before_game_wait = "none"
```

### `user-confirmation`

Starts the tool and keeps the game stopped while the user configures it.

```toml
launch = "before-game"
before_game_wait = "user-confirmation"
required = true
close_when_game_exits = true
```

On Windows, Tandem displays a foreground, topmost native OK/Cancel dialog.

- **OK:** continue to the game.
- **Cancel:** fail the session and close tools already started by the session.
- **Tool exits before confirmation:** evaluate its exit status using `required` and
  `continue_on_optional_tool_failure`.

A tool that remains open is governed by `close_when_game_exits` after the game starts.

This workflow does not depend on the trainer remaining visible after launch. Fullscreen,
native-rendering, or direct-scanout modes may cover or bypass secondary Windows windows.

### `tool-exit`

Waits for the tool to finish before starting the game.

```toml
launch = "before-game"
before_game_wait = "tool-exit"
required = true
```

Exit code `0` continues. A nonzero exit fails when `required = true` or when optional failures are
globally disallowed.

## Launch delays

`delay_ms` applies before the tool starts.

```toml
delay_ms = 2000
```

After-game delays are interruptible. If the game exits while Tandem is waiting, the delayed tool
and all remaining after-game tools are skipped.

## Required and optional tools

```toml
required = false
```

An optional tool may fail without blocking the game when
`continue_on_optional_tool_failure = true`.

```toml
required = true
```

A required tool must start successfully. A required `tool-exit` utility must also return exit code
`0`.

## Cleanup behavior

```toml
close_when_game_exits = true
```

Tandem attempts to terminate the direct child process it started after normal game exit.
Descendant processes created by a launcher, script, or tool are outside the current cleanup
guarantee.

## Path and file policy

With `allow_external_paths = false`, configured paths must remain under the directory containing
`Tandem.toml`.

Validation rejects:

- absolute or prefixed paths;
- `..` traversal;
- canonical paths that escape through symlinks or Windows junctions;
- program paths that resolve to directories;
- working directories that resolve to files;
- missing log-parent directories;
- log paths that escape the portable folder; and
- log paths that overwrite the configuration, game, or a configured tool.

Use `allow_external_paths = true` only when an external path is deliberate and trusted.

## BAT and CMD entries

Windows builds invoke BAT and CMD entries through a fixed:

```text
cmd.exe /D /S /C call ...
```

Arguments are supported and preserved. Tandem rejects embedded quotes, shell operators, expansion
characters, control characters, and other unsafe metacharacters in script paths or arguments.
There is no free-form shell-command field.

## Limits

- 32 configured tools
- 16 preparation steps per tool
- 256 non-control characters per window-title or control-class selector
- numeric control IDs from 1 to 2,147,483,647
- 2-minute maximum preparation wait
- 16 KiB combined argument text per program
- 10-minute maximum tool delay
- configuration version `1`

## Validate and preview

```text
TandemGameCompanion.exe --validate
TandemGameCompanion.exe --dry-run
```

`--validate` checks the configuration without launching anything. `--dry-run` prints the resolved
launch plan.
