# Planned controller support

> [!NOTE]
> Controller navigation is a design requirement for the future graphical configuration
> interface. It is not implemented in the current command-line alpha.

## Scope

Controller support should cover:

- Setup wizard navigation
- Buttons, toggles, lists, and sections
- Tool ordering through explicit Move Up and Move Down actions
- Tandem's own file browser
- Save, validate, test, cancel, and close actions
- Controller disconnect and reconnect

An on-screen keyboard is out of scope. Advanced text fields may require a physical keyboard
or platform-provided text input. Normal setup should minimize typing by deriving names from
selected filenames and using safe defaults.

## Default actions

| Input | Action |
|---|---|
| D-pad or left stick | Move focus |
| South button | Confirm or toggle |
| East button | Back or cancel |
| Left/right bumper | Previous/next section |
| Start/Menu | Primary action |
| View/Select | Diagnostics or help |

Bindings should be translated into semantic UI actions rather than scattered controller-button
checks.

## Focus requirements

- Every interactive control has a visible focus state.
- Focus order is deterministic.
- No operation requires hover, right-click, or drag-only interaction.
- Focus cannot disappear into a non-interactive element.

## Gameplay boundary

After the configuration interface closes, Tandem should stop controller polling and release
controller resources. Notifications must remain display-only and must not consume controller
input.
