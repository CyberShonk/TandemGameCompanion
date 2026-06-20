# Security model

## Goal

Tandem launches only local files explicitly named in its configuration and does not add
privileges or covert capabilities.

## Prohibited behavior

Tandem must not:

- Request administrator privileges or UAC elevation
- Install services, drivers, scheduled tasks, or startup entries
- Inject DLLs or manipulate another process's memory
- Download or update companion tools
- Open network listeners or contact remote services during launch
- Disable security software
- Hide itself as a system process
- Expose a raw, unrestricted shell-command field

## Implemented restrictions

The current configuration validator:

- Defaults to portable paths inside the configuration folder.
- Rejects absolute, prefixed, parent-traversal, and resolved external paths unless
  `allow_external_paths = true`.
- Rejects recursive attempts to launch Tandem itself.
- Restricts Windows entries to EXE, COM, BAT, and CMD files.
- Rejects BAT/CMD arguments and unsafe command-shell metacharacters in script paths.
- Caps configured tools, argument text, and launch delays.

## Script support

BAT and CMD files run with the current user's permissions and can perform any action available
to that user. Only configure scripts you have inspected and trust.

Tandem constructs the `cmd.exe` invocation internally. It does not provide a free-form command
field.

## Process communication

The worker currently reports the game process ID through a reserved line on its standard
output stream. The guardian then opens a synchronization-only handle to the reported process
on Windows.

This protocol is an early implementation boundary, not a trusted security channel. A dedicated
inherited pipe or equivalent private process-owned channel is planned.

## Cleanup boundary

`close_when_game_exits` terminates the direct child process Tandem started. It does not
currently guarantee termination of descendants created by launchers, scripts, or tools.

## User responsibility

Selected programs are outside Tandem's trust boundary. They retain the current user's normal
permissions, and Tandem does not sandbox them.
