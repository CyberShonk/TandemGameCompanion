<div align="center">

# Tandem Game Companion

**Portable companion-tool launching for Windows games**

Start a game, trainer, controller utility, setup program, or helper script as one supervised session.

[![Release](https://img.shields.io/github/v/release/CyberShonk/TandemGameCompanion?include_prereleases&label=release)](https://github.com/CyberShonk/TandemGameCompanion/releases)
[![Continuous integration](https://github.com/CyberShonk/TandemGameCompanion/actions/workflows/ci.yml/badge.svg)](https://github.com/CyberShonk/TandemGameCompanion/actions/workflows/ci.yml)
[![Rust 1.85 or newer](https://img.shields.io/badge/Rust-1.85%2B-000000?logo=rust)](Cargo.toml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[Releases](https://github.com/CyberShonk/TandemGameCompanion/releases) · [User Guide](docs/user-guide.md) · [Configuration](docs/CONFIGURATION.md) · [Troubleshooting](docs/troubleshooting.md) · [Changelog](CHANGELOG.md)

</div>

---

Tandem Game Companion is a portable Windows launcher that starts a game and its configured
companion tools together. It is intended for native Windows and Wine-based compatibility
environments such as GameNative and Winlator.

Tandem is useful when a game needs a trainer, controller utility, setup script, performance
tool, or another local helper launched in a predictable order.

> [!WARNING]
> Tandem is alpha software. The command-line launcher works, but the graphical setup interface,
> general controller navigation, notifications, and broad device compatibility testing are not
> complete.

## What it does

Tandem can:

- load and validate a portable `Tandem.toml` configuration;
- launch EXE, COM, BAT, and CMD entries;
- start tools before or after the game;
- delay tool launches when required;
- wait for a visible top-level window or visible enabled standard Win32 control owned by a launched before-game tool;
- pause for a native confirmation before starting the game;
- wait for a one-shot setup utility to finish;
- allow optional tool failures without blocking the game;
- supervise the configured game process;
- close selected tools when the game exits;
- preserve meaningful game, required-tool, and worker exit codes; and
- write a per-session `Tandem.log` for troubleshooting.

The guardian/worker process model keeps the outer Tandem process alive if the worker fails after
reporting the game process. Launched programs cannot write to or hold open the guardian status
channel.

## Common workflows

| Workflow | Configuration |
|---|---|
| Start a normal companion after the game | `launch = "after-game"` |
| Wait until a launched trainer window is ready | `[[tools.prepare]]` with `action = "wait-for-window"` |
| Wait until a standard trainer control is ready | `[[tools.prepare]]` with `action = "wait-for-control"` |
| Select a standard Win32 ComboBox item by index | `[[tools.prepare]]` with `action = "select-combo-box-index"` |
| Open a trainer, configure it, then continue | `before_game_wait = "user-confirmation"` |
| Run a setup utility and wait for success | `before_game_wait = "tool-exit"` |
| Prevent the game from starting without a tool | `required = true` |
| Close a tool when the game exits | `close_when_game_exits = true` |

See the [User Guide](docs/user-guide.md) for complete examples.

## Games and environments

Tandem is designed for:

- native Windows 10 and Windows 11;
- Wine-based desktop environments;
- GameNative;
- Winlator and related Android Windows compatibility environments; and
- portable game folders where the launcher, configuration, game, and tools can stay together.

Real-device GameNative and Winlator coverage is still limited. Compatibility should be treated as
experimental until a specific game, tool, device, and container configuration has been tested.

## Get started

1. Download the latest pre-release package from [Releases](https://github.com/CyberShonk/TandemGameCompanion/releases).
2. Extract it into the game folder.
3. Put companion programs in the included `Tools` folder.
4. Edit `Tandem.toml` so the paths match the game and tools.
5. Configure Windows, GameNative, Winlator, or Wine to launch `TandemGameCompanion.exe` instead of the game executable.
6. Start the game normally and check `Tandem.log` if anything fails.

A typical portable layout looks like this:

```text
GameFolder/
├── TandemGameCompanion.exe
├── Tandem.toml
├── Tandem.log
├── ExampleGame.exe
└── Tools/
    ├── Trainer.exe
    └── ControllerUtility.exe
```

Start with [`Tandem.example.toml`](Tandem.example.toml), then read the
[configuration reference](docs/CONFIGURATION.md).

### Wait for a trainer window

Use a preparation step when Tandem should wait until the exact tool process it launched owns a
visible top-level window:

```toml
[[tools]]
name = "Trainer"
path = "Tools/Trainer.exe"
launch = "before-game"
required = true
close_when_game_exits = true

[[tools.prepare]]
action = "wait-for-window"
title_contains = "Trainer"
timeout_ms = 10000
```

Window matching is restricted to the launched tool PID. Tandem does not focus, activate, move,
click, or otherwise modify the matched window.

### Wait for a standard Win32 control

Use this after the tool's parent window exists when the game should remain stopped until a specific
standard control becomes visible and enabled:

```toml
[[tools.prepare]]
action = "wait-for-control"
window_title_contains = "Trainer"
control_class_equals = "ComboBox"
control_id = 1001
timeout_ms = 10000
```

The parent top-level window and descendant control must belong to the exact tool PID Tandem
launched. When both `control_id` and `control_class_equals` are present, both must match. Tandem
does not read control text, select an option, click, focus, activate, send input, use UI Automation,
or modify the control. Custom-drawn controls without a standard descendant HWND are not supported by
this action.

### Select a standard Win32 ComboBox item

Use this narrowly scoped mutation when a directly launched before-game tool exposes a standard
Win32 `ComboBox` and the required choice has a stable zero-based index:

```toml
[[tools.prepare]]
action = "select-combo-box-index"
window_title_contains = "Trainer"
control_class_equals = "ComboBox"
control_id = 1001
selected_index = 2
timeout_ms = 10000
```

Tandem waits for one unambiguous visible parent and one unambiguous visible, enabled descendant
control owned by the exact launched PID. It verifies the runtime class, validates the item count,
reads the prior index, sets and verifies the requested index, sends one standard
`WM_COMMAND`/`CBN_SELCHANGE` parent notification when the value changed, and verifies the index
again. An already-selected index is a no-op and sends no notification. This action does not match
item text, open the list, focus or activate windows, send keyboard or mouse input, invoke other
controls, use UI Automation, or support custom-drawn controls.

### Before-game trainer confirmation

Use this when a trainer must be opened and configured before the game starts:

```toml
[[tools]]
name = "Trainer"
path = "Tools/Trainer.exe"
launch = "before-game"
before_game_wait = "user-confirmation"
required = true
close_when_game_exits = true
```

Tandem starts the trainer, keeps the game stopped, and displays a native OK/Cancel prompt. The
workflow does not depend on the trainer remaining visible after the game starts. Fullscreen,
native-rendering, or direct-scanout modes may obscure secondary Windows windows.

### Wait for a setup utility

Use this for a one-shot program that must finish successfully before the game starts:

```toml
[[tools]]
name = "Setup Utility"
path = "Tools/Setup.exe"
launch = "before-game"
before_game_wait = "tool-exit"
required = true
```

A required nonzero exit prevents game launch and becomes Tandem's exit status.

## Command-line options

```text
TandemGameCompanion.exe [OPTIONS]

-c, --config PATH    Use a configuration file other than Tandem.toml
    --validate       Validate the configuration without launching anything
    --dry-run        Print the resolved launch plan without launching anything
-h, --help           Show help
-V, --version        Show the application version
```

## Project status

Tandem is suitable for controlled alpha testing. Current limitations include:

- configuration is edited manually;
- normal launches use a visible console window;
- there is no graphical setup editor;
- general controller-driven configuration is not implemented;
- cleanup targets the directly launched process rather than an entire descendant process tree;
- worker recovery does not restart the worker or reconstruct tool cleanup after a crash; and
- BAT/CMD arguments are limited to values accepted by Tandem's fixed, validated `cmd.exe` invocation.

Known changes are recorded in the [Changelog](CHANGELOG.md).

## Documentation

| Document | Purpose |
|---|---|
| [Documentation Index](docs/index.md) | Starting point for all project documentation |
| [User Guide](docs/user-guide.md) | Normal setup and launch workflows |
| [Configuration Reference](docs/CONFIGURATION.md) | Complete TOML fields, validation, and examples |
| [Troubleshooting](docs/troubleshooting.md) | Common failures and likely fixes |
| [Architecture](docs/ARCHITECTURE.md) | Process model and source-module responsibilities |
| [Guardian and Worker](docs/GUARDIAN_WORKER.md) | Supervision behavior and recovery boundary |
| [Security Model](docs/SECURITY_MODEL.md) | Trust boundary and prohibited behavior |
| [Testing](docs/TESTING.md) | Automated and manual validation expectations |
| [Windows Testing](docs/WINDOWS_TESTING.md) | Windows build and Wine smoke-test instructions |
| [UX Requirements](docs/UX_REQUIREMENTS.md) | Planned graphical setup behavior |
| [Controller Support](docs/CONTROLLER_SUPPORT.md) | Planned controller-accessibility requirements |

## Building and testing

Tandem requires Rust 1.85 or newer.

```bash
./scripts/check-project.sh
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo test --all-targets --all-features -- --test-threads=1
cargo clippy --all-targets --all-features -- -D warnings
```

Build and smoke-test the Windows target from the configured Linux development environment:

```bash
./scripts/build-windows.sh
./scripts/test-windows.sh
```

See [Windows Testing](docs/WINDOWS_TESTING.md) for prerequisites and expected outputs.

## Security

Tandem runs selected programs with the permissions of the current user. Only configure games,
executables, and scripts from sources you trust.

Tandem does not request administrator privileges, install services, inject code, download tools,
establish persistence, or expose an unrestricted shell-command field. See the
[Security Policy](SECURITY.md) and [Security Model](docs/SECURITY_MODEL.md).

## Contributing

Bug reports, compatibility results, documentation corrections, focused code changes, and
reproducible tests are useful. Read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting a larger
change.

## License

Tandem Game Companion is released under the [MIT License](LICENSE).

## Disclaimer

Tandem Game Companion is an independent project. Users are responsible for following the
licenses and distribution terms for games, trainers, mods, scripts, and other third-party tools.
The project does not claim ownership of GameNative, Winlator, Wine, or any software launched
through Tandem.

## Standard Win32 button invocation

Tandem supports the bounded `invoke-button` preparation action for a uniquely matched, visible, enabled standard Win32 `Button` control owned by the directly launched tool process. The action requires a numeric `control_id`, accepts only `BS_PUSHBUTTON` or `BS_DEFPUSHBUTTON`, and sends one bounded `BM_CLICK`. It does not focus or activate windows, synthesize keyboard or mouse input, invoke checkboxes or radio buttons, discover descendant processes, or support custom-drawn controls.
