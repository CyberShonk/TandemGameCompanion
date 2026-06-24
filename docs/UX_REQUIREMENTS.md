# Planned Graphical Interface Requirements

[Documentation index](index.md) · [Controller Support](CONTROLLER_SUPPORT.md) · [User Guide](user-guide.md) · [Configuration](CONFIGURATION.md)

---

> [!NOTE]
> These requirements describe the intended configuration interface. The current alpha is
> command-line based and does not implement `--configure`.

## Primary workflow

A normal first-time setup should require only:

1. select the game executable;
2. add one or more optional companion executables or scripts;
3. choose launch timing when the default is unsuitable;
4. validate the configuration;
5. test the launch plan; and
6. save.

After setup, Tandem should launch silently by default.

## Interface structure

The interface should remain limited to four primary sections:

- **Game**
- **Tools**
- **Behavior**
- **Diagnostics**

Advanced settings should remain collapsed until requested.

The interface should not add an account system, marketplace, updater service, tray dependency,
dashboard, or unrelated system-management features.

## Intended defaults

- silent normal launch;
- errors remain visible while successful status is temporary;
- relative paths where possible;
- optional tool failures do not block the game;
- companion tools close with the game only when explicitly configured;
- the latest session log is retained with a bounded size; and
- portable safe mode remains enabled.

## Before-game setup

The graphical interface should expose the existing wait modes in plain language:

- **Start and continue** — `before_game_wait = "none"`
- **Wait for my confirmation** — `before_game_wait = "user-confirmation"`
- **Wait for the tool to close** — `before_game_wait = "tool-exit"`

Required-tool and cleanup behavior should be visible beside these choices rather than hidden in an
advanced screen.

## Recovery

A future `--configure` mode should let users reopen setup. An invalid or missing configuration
should return to first-run setup instead of leaving the user locked out of silent mode.

Configuration saves should use validate-then-write behavior and atomic replacement.

## Controller and text input

All normal setup actions should be controller-accessible. Text entry should not be required for the
normal path. Tool names should be derived from selected filenames, and safe defaults should avoid
manual arguments. No custom on-screen keyboard is planned.

## Notification behavior

Notifications should:

- avoid activating or minimizing the game;
- avoid consuming controller input;
- combine launch results into one concise status when possible;
- distinguish process creation from verified continued execution; and
- fall back to the log when they cannot be displayed reliably.
