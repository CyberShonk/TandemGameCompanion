# Configuration

Tandem reads a human-editable TOML file. The default filename is `Tandem.toml` in the current
working directory. Use `--config PATH` to select another file.

Start with [`../Tandem.example.toml`](../Tandem.example.toml).

## Current schema

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
name = "Companion Tool"
path = "Tools/CompanionTool.exe"
arguments = []
working_directory = "Tools"
enabled = true
launch = "after-game"
delay_ms = 2000
required = false
close_when_game_exits = true
```

## Top-level fields

| Field | Required | Meaning |
|---|---:|---|
| `config_version` | Yes | Must currently be `1` |
| `launcher` | No | Launcher-wide behavior; defaults are used when omitted |
| `game` | Yes | The primary program Tandem supervises |
| `tools` | No | Zero or more companion programs or scripts |

## Launcher fields

| Field | Default | Meaning |
|---|---|---|
| `log_file` | `Tandem.log` | Session-log output path |
| `allow_external_paths` | `false` | Allows absolute paths and paths outside the configuration folder |
| `continue_on_optional_tool_failure` | `true` | Continues when a non-required tool cannot start |

## Game and tool program fields

| Field | Required | Meaning |
|---|---:|---|
| `name` | Yes | Human-readable label used in output and logs |
| `path` | Yes | Program or script path |
| `arguments` | No | Argument array; defaults to empty |
| `working_directory` | No | Existing folder used as the child process working directory; defaults to the selected file's parent folder |

## Tool-only fields

| Field | Default | Meaning |
|---|---|---|
| `enabled` | `true` | Skips the tool when set to `false` |
| `launch` | `after-game` | Accepts `before-game` or `after-game` |
| `delay_ms` | `0` | Delay before launch; maximum 600,000 milliseconds |
| `required` | `false` | Treats launch failure as a session failure |
| `close_when_game_exits` | `false` | Terminates the directly launched child process after the game exits |

## Path policy

With `allow_external_paths = false`, configured paths must remain inside the directory that
contains the selected configuration file. Validation rejects:

- Absolute paths
- Windows path prefixes and UNC paths
- Parent-directory traversal such as `../Tool.exe`
- Symlink-resolved paths outside the portable folder
- Attempts to launch the running Tandem executable recursively

The selected game, tools, and working directories must already exist. The log file itself may
be created when the session starts.

## Windows entry types

Windows builds accept these extensions:

- `.exe`
- `.com`
- `.bat`
- `.cmd`

BAT and CMD entries are invoked through `cmd.exe`. Custom arguments for scripts are currently
rejected, and script paths containing command-shell metacharacters are rejected.

## Limits

- Maximum configured tools: 32
- Maximum combined argument text per program: 16 KiB
- Maximum tool delay: 10 minutes
- Supported configuration version: 1

## Validation commands

```text
TandemGameCompanion.exe --validate
TandemGameCompanion.exe --dry-run
```

`--dry-run` resolves and prints canonical paths. Review its output before sharing logs because
those paths may identify local folders.
