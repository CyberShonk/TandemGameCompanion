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
- Preparation recipes are allowlisted, count-limited, and bounded; mutations are limited to standard Win32 ComboBox index selection, push-button invocation, `BS_AUTOCHECKBOX` state setting, `BS_AUTORADIOBUTTON` selection, and standard single-line editable `Edit` text setting.
- Top-level window discovery accepts only visible windows owned by the exact launched tool PID.
- Control discovery is restricted to visible, enabled descendant HWNDs under the selected top-level
  window and owned by that same PID.
- Control selection uses only a numeric control ID, exact Win32 class name, or both with AND
  semantics. Control text is never used for discovery. `set-edit-text` reads text only after the
  process, parent window, control ID/class, visibility, enabled state, and runtime class are resolved.
- ComboBox selection uses only documented `CB_GETCOUNT`, `CB_GETCURSEL`, and `CB_SETCURSEL`
  messages plus one standard `WM_COMMAND`/`CBN_SELCHANGE` notification when the index changes.
- Push-button invocation accepts only `BS_PUSHBUTTON`/`BS_DEFPUSHBUTTON` and sends one bounded
  `BM_CLICK`. Auto-checkbox state setting accepts only `BS_AUTOCHECKBOX`, reads with bounded
  `BM_GETCHECK`, skips mutation when already correct, otherwise sends one bounded `BM_CLICK`, and
  verifies the resulting state. Auto-radio selection accepts only `BS_AUTORADIOBUTTON`, reads with
  bounded `BM_GETCHECK`, skips mutation when already selected, otherwise sends one bounded `BM_CLICK`,
  and requires the target to become checked without directly rewriting sibling states. Edit text
  setting accepts only standard single-line editable `Edit`
  controls, reads with bounded `WM_GETTEXTLENGTH`/`WM_GETTEXT`, sends at most one bounded
  `WM_SETTEXT`, and requires an exact read-back result.
- Mutations require one unambiguous parent and control, the expected runtime class and style,
  visible/enabled state, operation-specific before/after verification, and a directly launched PID.
- Preparation performs no activation, focus, movement, keyboard or mouse input, text-based control
  discovery, arbitrary configurable messages, multiline/password/read-only/text-transforming Edit
  mutation, manual-radio or unsupported Button-style mutation, direct sibling-radio rewriting, UI
  Automation, image matching, custom-control mutation, or descendant-process following.
- Window and control preparation reject BAT/CMD wrappers and do not follow descendant processes.
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
