# Planned Controller Support

[Documentation index](index.md) · [UX Requirements](UX_REQUIREMENTS.md) · [User Guide](user-guide.md) · [Troubleshooting](troubleshooting.md)

---

> [!NOTE]
> Controller navigation is a requirement for the future graphical configuration interface. It is
> not implemented in the current command-line alpha.

The native before-game confirmation dialog is already usable through normal Windows input. Touch
usually works directly in compatible environments. Controller use depends on the environment's
controller-to-pointer or keyboard mapping.

## Scope

Future controller support should cover:

- setup-wizard navigation;
- buttons, toggles, lists, and sections;
- tool ordering through explicit **Move Up** and **Move Down** actions;
- Tandem's own file browser;
- save, validate, test, cancel, and close actions; and
- controller disconnect and reconnect.

An on-screen keyboard is out of scope. Advanced text fields may require a physical keyboard or
platform-provided text input. Normal setup should minimize typing by deriving names from selected
files and using safe defaults.

## Default actions

| Input | Action |
|---|---|
| D-pad or left stick | Move focus |
| South button | Confirm or toggle |
| East button | Back or cancel |
| Left/right bumper | Previous or next section |
| Start/Menu | Primary action |
| View/Select | Diagnostics or help |

Bindings should map to semantic UI actions rather than scattered controller-button checks.

## Focus requirements

- Every interactive control has a visible focus state.
- Focus order is deterministic.
- No operation requires hover, right-click, or drag-only interaction.
- Focus cannot disappear into a non-interactive element.
- Dialogs return focus to a predictable control when closed.

## Gameplay boundary

After the configuration interface closes, Tandem should stop controller polling and release its
controller resources. Notifications must remain display-only and must not consume gameplay input.
