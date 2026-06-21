# Security model

Tandem launches only local files explicitly named in configuration. It does not elevate,
install persistence, inject code, download tools, open listeners, disable security software,
or expose a raw shell-command field.

## Implemented restrictions

- Portable paths are the default; absolute, prefixed, parent-traversal, and canonical external
  paths are rejected unless `allow_external_paths = true`.
- File and directory types are checked, recursive Tandem launch is rejected, and tool/delay/
  argument limits are enforced.
- Existing log targets and parents are canonicalized to stop symlink/junction escapes. The log
  cannot overwrite the configuration, game, or a configured tool.
- Windows entries are limited to EXE, COM, BAT, and CMD files.
- BAT/CMD paths and arguments are validated before Tandem constructs its fixed `cmd.exe`
  invocation; free-form command text is not accepted.
- Child output is written to the session log. The Windows guardian status handle is marked
  non-inheritable before games or tools are created.

BAT/CMD files and selected executables still run with the current user's full permissions.
Only use files you trust.

`close_when_game_exits` targets the direct child Tandem launched. Descendants created by a
launcher or script are outside the current cleanup guarantee.
