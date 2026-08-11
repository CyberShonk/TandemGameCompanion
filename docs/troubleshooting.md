# Tandem Game Companion Troubleshooting

[Documentation](index.md) · [User Guide](user-guide.md) · [Configuration](CONFIGURATION.md)

Start with `Tandem.log`. The last successful step usually identifies where the session stopped.

## Tandem cannot find `Tandem.toml`

Check that `Tandem.toml` is beside `TandemGameCompanion.exe` and that the environment working directory points to that folder.

Use `--config PATH` only when you deliberately keep the configuration somewhere else.

## A path is rejected

Common causes are:

- the file does not exist;
- a program path points to a directory;
- a working directory points to a file;
- the path contains parent traversal;
- an external path is used while `allow_external_paths = false`;
- a link or junction resolves outside the portable folder; or
- the log path overlaps another configured file.

Use relative paths when possible and run `TandemGameCompanion.exe --validate` again.

## The game does not start

Check the final lines of `Tandem.log` for a required tool failure, a failed preparation step, a required setup utility returning nonzero, confirmation cancellation, an invalid game path, or a process creation error.

Use `--dry-run` to confirm the resolved paths and arguments without launching anything.

## A preparation step times out

Check the configured window title, control ID, class name, timeout, and tool process in `Tandem.log`.

Preparation only matches the directly launched tool process. A launcher that exits after creating a different GUI process is outside the current boundary.

Custom drawn interfaces may not expose the standard Win32 windows or controls required by a preparation action.

### `wait-for-window`

The window title match is case sensitive. Use `title_contains` when a stable title fragment is more reliable than the complete title.

### `wait-for-control`

The control must be visible, enabled, under the selected top level window, and owned by the directly launched tool process. When both an ID and class are configured, both must match.

## A mutating preparation action fails

| Action | Required target |
|---|---|
| `select-combo-box-index` | Standard `ComboBox` with a valid requested zero based index |
| `invoke-button` | Standard `BS_PUSHBUTTON` or `BS_DEFPUSHBUTTON` |
| `set-checkbox-state` | Standard `BS_AUTOCHECKBOX` |
| `select-radio-button` | Standard `BS_AUTORADIOBUTTON` |
| `set-edit-text` | Standard visible enabled single line editable `Edit` |

All mutating actions require one unambiguous parent window and one unambiguous visible enabled control owned by the directly launched tool process.

If the target is hidden, disabled, the wrong runtime class, the wrong control style, ambiguous, or in another process, Tandem rejects it.

For `set-edit-text`, multiline, password, read only, case transforming, and OEM transforming controls are also rejected. Configured text may not exceed 4,096 UTF-16 units or contain NUL, carriage return, or line feed.

## A required setup utility stops the session

With:

```toml
before_game_wait = "tool-exit"
required = true
```

any nonzero exit code prevents game launch. Check the logged exit code and test the utility by itself.

## The confirmation dialog is not visible or usable

The dialog appears before game launch.

Confirm that the tool uses `launch = "before-game"` and `before_game_wait = "user-confirmation"`. Check whether another mapped window covers the dialog. Touch and controller behavior depend on the compatibility environment's normal input mapping.

## The trainer disappears behind the game

This can be normal. Fullscreen or native rendering modes may cover secondary Windows windows. Complete trainer setup before confirming game launch.

## A delayed tool did not launch

Tandem skips a delayed after game tool if the game exits before the delay finishes.

Also check whether the configured game launches a replacement process and exits immediately.

## A tool remains open after the game exits

Confirm:

```toml
close_when_game_exits = true
```

Tandem terminates the direct process it started. A separate descendant process created by a launcher or tool is outside the current cleanup guarantee.

## A BAT or CMD entry is rejected

Tandem accepts BAT and CMD files through a fixed `cmd.exe` invocation and rejects unsafe command text such as shell operators, expansion characters, embedded quotes, and control characters.

Use a trusted script file with simple validated arguments instead of arbitrary shell syntax.

## The container closes while the game should still run

Tandem can remain active while the configured game process runs, but it cannot prevent a compatibility environment from terminating the entire Wine session or container.

Check whether the game launches a replacement process and review the container's shutdown behavior.

## `Tandem.log` is missing

Return to the default while testing:

```toml
[launcher]
log_file = "Tandem.log"
```

Confirm the log parent directory exists and the path does not resolve outside the allowed folder or overlap another configured file.

## Reporting a problem

Include:

- Tandem release or commit;
- operating system or Android version;
- GameNative, Winlator, or Wine version when applicable;
- device model when applicable;
- game and tool names;
- exact reproduction steps;
- a sanitized `Tandem.toml`; and
- the relevant part of `Tandem.log`.

Remove credentials, unnecessary personal paths, account information, and unrelated log content before sharing. Do not upload copyrighted game files or proprietary third party executables.
