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

- a required tool launch failure;
- a required `tool-exit` utility returning nonzero;
- cancellation of the confirmation dialog;
- an invalid game path or working directory; or
- a process-creation error from Windows or Wine.

Use `--dry-run` to confirm the resolved game path and arguments without launching anything.

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
