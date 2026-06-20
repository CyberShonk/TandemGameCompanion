# Planned graphical-interface requirements

> [!NOTE]
> These requirements describe the intended configuration interface. The current alpha is
> command-line only and does not implement `--configure`.

## Primary workflow

A normal first-time configuration should require only:

1. Select the game executable.
2. Add one or more optional companion executables or scripts.
3. Choose launch timing when the default is unsuitable.
4. Validate, test, and save.

After setup, Tandem should launch silently by default.

## Interface structure

The configuration interface should remain limited to four primary sections:

- Game
- Tools
- Behavior
- Diagnostics

Advanced settings should remain collapsed until requested. The interface should not include an
account system, marketplace, updater service, tray dependency, dashboard, or unrelated
system-management features.

## Intended defaults

- Silent normal launch
- Errors remain visible; successful status is temporary
- Relative paths when possible
- Optional tool failures do not block the game
- Companion tools close with the game only when explicitly configured
- Latest-session log retained with bounded size
- Portable safe mode enabled

## Recovery

A future `--configure` mode should let users reopen setup. An invalid or missing configuration
should return to first-run setup rather than leaving the user locked out of silent mode.

Configuration saves should use validate-then-write behavior and atomic replacement.

## Controller and text input

All normal setup actions should be controller-accessible. Text entry should not be required
for the normal path. Tool names should be derived from filenames and sensible defaults should
avoid manual arguments. No on-screen keyboard is planned.

## Notification behavior

Notifications should:

- Avoid activating or minimizing the game.
- Avoid consuming controller input.
- Combine launch results into one concise status when possible.
- Distinguish process creation from verified continued execution.
- Fall back to the log when they cannot be displayed reliably.
