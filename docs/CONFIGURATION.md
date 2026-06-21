# Configuration

Tandem reads a TOML file. The default is `Tandem.toml` in the current working directory; use
`--config PATH` to select another file. Start with [`../Tandem.example.toml`](../Tandem.example.toml).

## Configuration compatibility

The configuration schema remains version `1`. Configurations created for `v0.1.0-alpha`
continue to work unchanged because `before_game_wait` defaults to `none` when omitted.

## Schema

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

## Fields

| Section | Field | Default | Meaning |
|---|---|---:|---|
| top level | `config_version` | required | Must currently be `1` |
| launcher | `log_file` | `Tandem.log` | Session-log output path |
| launcher | `allow_external_paths` | `false` | Permits paths outside the configuration folder |
| launcher | `continue_on_optional_tool_failure` | `true` | Continues after a non-required tool cannot start or, for a waited tool, exits unsuccessfully |
| game/tool | `name` | required | Label used in output and logs |
| game/tool | `path` | required | Program or BAT/CMD script path |
| game/tool | `arguments` | `[]` | Argument array |
| game/tool | `working_directory` | program parent | Existing child working directory |
| tool | `enabled` | `true` | Omits the tool when false |
| tool | `launch` | `after-game` | `before-game` or `after-game` |
| tool | `before_game_wait` | `none` | `none`, `user-confirmation`, or `tool-exit`; valid only for `before-game` tools |
| tool | `delay_ms` | `0` | Delay before launch, up to 600,000 ms |
| tool | `required` | `false` | Makes launch failure, or a waited nonzero exit, fail the session |
| tool | `close_when_game_exits` | `false` | Terminates the directly launched child after normal game exit |

## Before-game waiting

`before_game_wait = "user-confirmation"` starts the tool and keeps the game stopped while the
user configures it. On Windows, Tandem displays a foreground, topmost native OK/Cancel dialog.
OK continues to the game; Cancel fails the session and closes every tool started by that session.
If the tool exits before confirmation completes, its exit status is evaluated using `required`
and `continue_on_optional_tool_failure`. A tool that remains open is governed by
`close_when_game_exits` after the game starts.

This workflow does not rely on the trainer remaining visible after game launch. Fullscreen,
native-rendering, or direct-scanout modes may cover or bypass secondary Windows windows.

`before_game_wait = "tool-exit"` waits for the tool to finish. A zero exit continues. A nonzero
exit fails when `required = true` or when optional failures are globally disallowed.

`before_game_wait = "none"` preserves the original behavior: start the before-game tool and
continue immediately.

After-game delays are polled in short intervals. If the game exits during a delay, Tandem skips
that tool and all remaining after-game tools.

## Path and file policy

With `allow_external_paths = false`, paths must remain under the configuration directory.
Validation rejects absolute/prefixed paths, `..` traversal, and canonical paths that escape
through symlinks or Windows junctions. Program paths must be files; working directories must be
directories. The log parent directory must already exist, and the log may not resolve outside
the portable folder or overwrite the configuration, game, or a configured tool.

## BAT and CMD entries

Windows builds invoke BAT/CMD entries through a fixed `cmd.exe /D /S /C call ...` command.
Arguments are supported and preserved. Tandem rejects double quotes, shell operators,
expansion characters, control characters, and other unsafe metacharacters in script paths or
arguments rather than exposing a free-form shell command.

## Limits and validation

- 32 configured tools
- 16 KiB combined argument text per program
- 10-minute maximum tool delay
- Configuration version `1`

```text
TandemGameCompanion.exe --validate
TandemGameCompanion.exe --dry-run
```
