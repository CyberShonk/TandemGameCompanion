# Tandem Game Companion

Tandem Game Companion is an experimental portable Windows launcher that starts a game and
its configured companion tools as one supervised session.

It is intended for native Windows and Wine-based compatibility environments such as
GameNative and Winlator, where a game may need a trainer, controller utility, setup script,
or other local helper launched alongside it.

> [!WARNING]
> Tandem is early alpha software. The command-line launcher works, but the graphical setup
> interface, controller navigation, notifications, and broad device compatibility testing
> are not complete.

## Current capabilities

- Loads and validates a versioned `Tandem.toml` configuration.
- Launches local EXE and COM programs.
- Launches explicitly configured BAT and CMD scripts through `cmd.exe` on Windows.
- Starts tools before or after the game with optional delays.
- Can pause before game launch for a native user confirmation or until a setup tool exits.
- Allows optional tool failures without blocking the game.
- Waits for the configured game process to exit.
- Optionally terminates directly launched tools when the game exits.
- Writes a per-session log and redirects child stdout/stderr away from guardian control messages.
- Uses a guardian/worker process model so the outer process remains alive if the worker
  fails after reporting the game process.
- Supports configuration validation and launch-plan previews without starting programs.

## Current limitations

- Configuration is edited manually; there is no graphical setup editor.
- Normal launches use a visible console window.
- General controller-driven configuration is not implemented.
- Worker recovery does not restart the worker or restore tool cleanup after a worker crash.
- Tool cleanup targets the directly launched process, not an entire descendant process tree.
- BAT/CMD arguments are limited to values that are safe for Tandem's fixed `cmd.exe` invocation.
- Native Windows, GameNative, and Winlator coverage is still limited.

See [`docs/GUARDIAN_WORKER.md`](docs/GUARDIAN_WORKER.md) for the current supervision boundary.

## Quick start

1. Build or obtain `TandemGameCompanion.exe`.
2. Place it beside a `Tandem.toml` configuration.
3. Keep the configured game and tools inside the same portable folder unless external paths
   are deliberately enabled.
4. Run `TandemGameCompanion.exe` instead of the game executable.

Start with [`Tandem.example.toml`](Tandem.example.toml) and read the complete
[configuration reference](docs/CONFIGURATION.md).

Existing `v0.1.0-alpha` configurations remain valid. The new `before_game_wait` field defaults
to `none` when omitted.

### Before-game setup workflows

Use user confirmation when a trainer must remain open while the user changes settings:

```toml
[[tools]]
name = "Trainer"
path = "Tools/Trainer.exe"
launch = "before-game"
before_game_wait = "user-confirmation"
required = true
close_when_game_exits = true
```

Tandem starts the trainer, keeps the game stopped, and displays a native OK/Cancel prompt.
The game starts only after OK. Cancel or a later game-launch failure cleans up tools already
started by the session.

Use tool-exit waiting for a one-shot setup utility:

```toml
[[tools]]
name = "Setup Utility"
path = "Tools/Setup.exe"
launch = "before-game"
before_game_wait = "tool-exit"
required = true
```

Tandem waits for the utility to exit. A required nonzero exit prevents game launch and is
returned as a nonzero Tandem exit status.

The confirmation workflow is intentionally completed before the game starts. Fullscreen,
native-rendering, or direct-scanout modes may obscure secondary Windows windows after launch,
so Tandem does not depend on the trainer remaining visible over the game.

### Command-line options

```text
TandemGameCompanion.exe [OPTIONS]

-c, --config PATH    Use a configuration file other than Tandem.toml
    --validate       Validate the configuration without launching anything
    --dry-run        Print the resolved launch plan without launching anything
-h, --help           Show help
-V, --version        Show the application version
```

## Building

Tandem requires Rust 1.85 or newer.

```bash
cargo fmt --check
cargo check --all-targets --all-features
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

Linux development environments can build the Windows executable with:

```bash
./scripts/build-windows.sh
```

The output is written to:

```text
target/windows-release/TandemGameCompanion.exe
target/windows-release/TandemGameCompanion.exe.sha256
```

See [`docs/WINDOWS_TESTING.md`](docs/WINDOWS_TESTING.md) for prerequisites and the Wine
smoke test.

## Repository documentation

- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — implemented process and module structure
- [`docs/CONFIGURATION.md`](docs/CONFIGURATION.md) — current TOML schema and validation rules
- [`docs/TESTING.md`](docs/TESTING.md) — automated checks and compatibility matrix
- [`docs/SECURITY_MODEL.md`](docs/SECURITY_MODEL.md) — trust boundary and prohibited behavior
- [`docs/UX_REQUIREMENTS.md`](docs/UX_REQUIREMENTS.md) — planned graphical interface behavior
- [`docs/CONTROLLER_SUPPORT.md`](docs/CONTROLLER_SUPPORT.md) — planned controller requirements

## Security

Tandem runs selected programs with the permissions of the current user. Only configure games,
executables, and scripts from sources you trust.

Tandem does not request administrator privileges, install services, inject code, download
third-party tools, establish persistence, or expose a raw shell-command field. See
[`SECURITY.md`](SECURITY.md) and [`docs/SECURITY_MODEL.md`](docs/SECURITY_MODEL.md).

## Development status

The current implementation is suitable for controlled alpha testing, not general release.
Known changes are recorded in [`CHANGELOG.md`](CHANGELOG.md).

## License

Tandem Game Companion is available under the [MIT License](LICENSE).
