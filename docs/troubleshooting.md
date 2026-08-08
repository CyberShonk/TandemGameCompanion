# Tandem Game Companion Troubleshooting

[Documentation index](index.md) · [User Guide](user-guide.md) · [Configuration](CONFIGURATION.md) · [Testing](TESTING.md)

---

Start with `Tandem.log`. It usually shows the last successful step before the failure.

## Tandem cannot find `Tandem.toml`

Possible causes:

- Tandem was started from the wrong working directory.
- The file was renamed or placed in another folder.
- The container points at the executable but uses a different working directory.

Try:

1. Place `Tandem.toml` beside `TandemGameCompanion.exe`.
2. Set the container working directory to that folder.
3. Run `TandemGameCompanion.exe --config PATH` only when deliberately using another location.

## A path is rejected during validation

Possible causes:

- the file does not exist;
- a program path points to a directory;
- a working directory points to a file;
- the path contains `..` traversal;
- an absolute path is used while external paths are disabled;
- a symlink or Windows junction resolves outside the portable folder; or
- the log path overlaps the configuration, game, or a tool.

Try:

1. Confirm the exact filename and extension.
2. Use a relative path with forward slashes.
3. Keep the game and tools under the configuration folder.
4. Leave `allow_external_paths = false` unless an external location is necessary.
5. Run `--validate` again.

## The game does not start

Check the final lines of `Tandem.log` for:

- a required tool launch or preparation failure;
- a required `tool-exit` utility returning nonzero;
- cancellation of the confirmation dialog;
- an invalid game path or working directory; or
- a process-creation error from Windows or Wine.

Use `--dry-run` to confirm the resolved game path and arguments without launching anything.

## A `wait-for-window` preparation step times out

Check `Tandem.log` for the configured selector, timeout, and launched PID.

Possible causes:

- the window title does not match exactly, including capitalization;
- the tool opens no visible top-level window;
- the configured timeout is too short;
- a launcher creates the actual GUI in a different process; or
- the tool is a BAT/CMD wrapper rather than a directly launched EXE or COM file.

Use `title_contains` for a stable title fragment when the full title includes a version or status.
Do not rely on another process with the same title; Tandem deliberately ignores it.

## A `wait-for-control` preparation step times out

Check the parent-window selector, control ID, exact class capitalization, timeout, and launched PID in
`Tandem.log`.

Tandem accepts only a visible, enabled descendant HWND under a matching visible top-level window
owned by the exact launched-tool PID. A matching control is deliberately ignored when it belongs to
another process, sits under a different top-level window, is hidden, is disabled, or satisfies only
one half of a combined ID/class selector.

Standard controls expose class names such as `ComboBox` or `Button`. Custom-drawn interfaces may not
expose a normal descendant control and cannot be detected by this action. Tandem does not inspect
control text, use UI Automation, or match images.

## A `select-combo-box-index` preparation step fails

Check the deterministic selector, requested index, prior/resulting index, and failure reason in
`Tandem.log`. The parent must be unambiguous, the descendant must be visible and enabled, the actual
runtime class must be exactly `ComboBox`, the numeric ID must fit the standard `WM_COMMAND` control
ID field, and the item count must include the zero-based requested index.

An unavailable index is polled only until the configured bounded timeout. Tandem does not match item
text, open the list, focus the control, send input, or support custom-drawn ComboBoxes. A tool that
exits and leaves its GUI in another process is outside the direct-PID boundary. An already-selected
index is successful without mutation or parent notification.

## A required setup utility stops the session

With:

```toml
before_game_wait = "tool-exit"
required = true
```

any nonzero utility exit prevents game launch. Check the logged exit code and run the utility by
itself to determine why it failed.

Use `required = false` only when the game can still run correctly without that utility.

## The confirmation dialog is not visible or usable

The dialog appears before the game starts and is a standard Windows dialog.

Try:

1. Confirm the tool uses `launch = "before-game"`.
2. Confirm `before_game_wait = "user-confirmation"`.
3. Check whether another mapped window is covering the dialog.
4. Test touch input directly.
5. Review the container's controller-to-pointer or keyboard mapping for controller use.

The dialog is not intended to remain present after the game launches.

## The trainer disappears behind the game

This can be normal. A fullscreen game may cover the trainer through normal window ordering. Native
rendering or direct scanout may bypass secondary X-server windows entirely.

Complete trainer setup before selecting **OK**. Do not depend on the trainer remaining visible over
the game.

## A delayed tool did not launch

Tandem skips delayed after-game tools when the game exits before the delay completes. This prevents
a tool from appearing after the game session has already ended.

Check whether the game exited early or spawned a replacement process that Tandem was not configured
to supervise.

## A tool remains open after the game exits

Check:

```toml
close_when_game_exits = true
```

Tandem terminates the direct process it launched. A tool may create another process and then exit;
that descendant is outside the current cleanup boundary.

Whenever possible, point Tandem directly at the persistent tool process rather than a short-lived
launcher.

## A BAT or CMD entry is rejected

Tandem supports BAT and CMD entries through a fixed `cmd.exe` invocation. It rejects unsafe command
text, including shell operators, expansion characters, embedded quotes, and control characters.

Do not work around this with arbitrary shell syntax. Use a trusted script file with simple,
validated arguments, or replace the workflow with a direct executable.

## The container closes even though the guardian is running

Tandem can keep its guardian process alive while the configured game runs. It cannot prevent a
compatibility environment from terminating the entire Wine session or container.

Check container-level shutdown settings and whether the game launches a different replacement
process.

## `Tandem.log` is missing

Possible causes:

- the log parent directory does not exist;
- the configured log path is invalid;
- the log resolves outside the portable folder;
- the log overlaps another configured file; or
- Tandem failed before opening the log.

Return to the default while testing:

```toml
[launcher]
log_file = "Tandem.log"
```

## Reporting a problem

Include:

- Tandem release or commit;
- operating system or Android version;
- GameNative, Winlator, or Wine version;
- device model when applicable;
- game and tool names;
- a sanitized `Tandem.toml`;
- the relevant `Tandem.log`; and
- exact reproduction steps.

Do not upload credentials, copyrighted game files, or proprietary third-party executables.

## Standard Win32 button invocation

Tandem supports the bounded `invoke-button` preparation action for a uniquely matched, visible, enabled standard Win32 `Button` control owned by the directly launched tool process. The action requires a numeric `control_id`, accepts only `BS_PUSHBUTTON` or `BS_DEFPUSHBUTTON`, and sends one bounded `BM_CLICK`. It does not focus or activate windows, synthesize keyboard or mouse input, invoke checkboxes or radio buttons, discover descendant processes, or support custom-drawn controls.
