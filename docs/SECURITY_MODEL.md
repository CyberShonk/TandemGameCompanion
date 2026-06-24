# Tandem Game Companion Security Model

[Documentation index](index.md) · [Security Policy](../SECURITY.md) · [Configuration](CONFIGURATION.md) · [Architecture](ARCHITECTURE.md)

---

## Goal

Tandem launches only local files explicitly named in its configuration. It does not add privileges
or hidden system capabilities.

## Prohibited behavior

Tandem must not:

- request administrator privileges or UAC elevation;
- install services, drivers, scheduled tasks, or startup entries;
- inject DLLs or manipulate another process's memory;
- download or update companion tools;
- open network listeners during launch;
- disable security software;
- hide itself as a system process; or
- expose a raw, unrestricted shell-command field.

## Implemented restrictions

- Portable paths are the default.
- Absolute, prefixed, parent-traversal, and resolved external paths are rejected unless
  `allow_external_paths = true`.
- Program paths must resolve to files and working directories must resolve to directories.
- Recursive Tandem launch is rejected.
- Tool-count, argument-size, and delay limits are enforced.
- Existing log targets and parent directories are canonicalized to stop symlink or junction
  escapes.
- Dangling log links are rejected.
- The log cannot overwrite the configuration, game, or a configured tool.
- Windows entries are limited to EXE, COM, BAT, and CMD files.
- BAT/CMD paths and arguments are validated before Tandem constructs its fixed `cmd.exe`
  invocation.
- Free-form command text is not accepted.
- Child output is written to the session log rather than the guardian status channel.
- The Windows guardian status handle is marked non-inheritable before games or tools are created.

## Script support

BAT and CMD files run with the current user's permissions and can perform any action available to
that user. Only configure scripts you have inspected and trust.

Tandem supports simple validated arguments. It rejects shell operators, expansion characters,
embedded quotes, control characters, and other unsafe command text rather than attempting to
sanitize an arbitrary shell command.

## Process communication

The worker reports one game PID through a reserved status record. Child output redirection and
Windows handle-inheritance protection prevent launched games and tools from impersonating this
record through inherited stdout handles.

The worker itself remains inside Tandem's trusted process boundary. A future private anonymous pipe
could reduce protocol coupling further, but it is not required for ordinary child isolation.

## Cleanup boundary

`close_when_game_exits` terminates the direct child Tandem started. It does not guarantee
termination of descendants created by launchers, scripts, or tools.

## User responsibility

Selected programs remain outside Tandem's trust boundary. They retain the current user's normal
permissions, and Tandem does not sandbox them.
