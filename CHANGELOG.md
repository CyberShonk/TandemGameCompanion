# Changelog

Notable user visible changes to Tandem Game Companion are recorded here.

## Unreleased

### Added

- Ordered per tool preparation recipes.
- `wait-for-window` and `wait-for-control` readiness checks scoped to the directly launched tool process.
- Standard Win32 ComboBox selection by zero based index.
- Standard push button invocation.
- Standard automatic checkbox state setting.
- Standard automatic radio button selection.
- Standard single line Edit text setting with content redaction from preparation and result logs.
- Wine smoke coverage for supported preparation actions, isolation, no op behavior, invalid targets, timeouts, tool exit, and required or optional failure handling.

### Changed

- Public documentation is consolidated around setup, configuration, troubleshooting, security, contribution, and release history.
- Tool preparation documentation now describes supported user behavior and limits without duplicating internal implementation details.

### Security

- Window and control preparation is restricted to the directly launched tool process.
- Mutating preparation actions require supported standard Win32 control types, unambiguous targets, bounded operations, and action specific verification.
- Arbitrary configurable Windows messages, synthetic keyboard or mouse input, UI Automation, image matching, and unrestricted shell commands are not exposed.

## [0.2.0-alpha] - 2026-06-24

### Added

- General `before_game_wait` modes for native user confirmation and one shot setup utilities.
- A native Windows OK/Cancel prompt for tools that must be configured before game launch.
- Lifecycle tests for early failures, persistent tools, delayed launches, worker failure, and status channel isolation.
- User guide, troubleshooting guide, and documentation index.

### Changed

- Existing configurations remain compatible because omitted `before_game_wait` values default to `none`.
- BAT and CMD game and tool entries preserve validated arguments through Tandem's fixed `cmd.exe` invocation.

### Fixed

- Prevented launched games and tools from inheriting or writing to the guardian status channel.
- Cleaned up started tools when game launch or another controlled session step fails.
- Preserved meaningful game, required tool, and worker exit codes.
- Stopped delayed after game tools when the game exits first.
- Strengthened path and log destination validation.

## [0.1.0-alpha] - 2026-06-20

### Added

- Functional Rust launcher with versioned TOML configuration.
- EXE, COM, BAT, and CMD launch support on Windows.
- Before game and after game companion tool sequencing.
- Optional delays, required tool behavior, and direct child process cleanup.
- Guardian and worker process separation with game process supervision.
- Configuration validation and dry run modes.
- Session logging.
- Linux quality checks, Windows MSVC build coverage, and an isolated Wine smoke test.
- Alpha packaging script and tester instructions.

### Known limitations

- No graphical configuration interface or general controller driven setup.
- No worker restart or cleanup reconstruction after worker failure.
- Limited real device validation in GameNative and Winlator.
