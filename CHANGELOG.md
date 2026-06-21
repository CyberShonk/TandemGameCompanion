# Changelog

All notable changes to Tandem Game Companion will be documented here.

## Unreleased

### Added

- General-purpose `before_game_wait` modes for native user confirmation and one-shot setup utilities.
- Lifecycle integration tests for early failures, persistent tools, delayed launches, guardian recovery, and guardian-protocol spoof attempts.

### Fixed

- Prevented games and tools from inheriting or writing to the guardian status channel.
- Ensured started tools are cleaned up on game-launch and other worker failure paths.
- Preserved game, required-tool, and worker exit codes.
- Made after-game delays stop when the game exits.
- Validated program/working-directory types and protected log paths from symlink or junction escapes and destructive file overlap.
- Passed validated arguments to BAT/CMD game and tool entries.
- Made generated SHA-256 records portable by recording only the executable filename.

## [0.1.0-alpha] - 2026-06-20

### Added

- Functional Rust launcher with versioned TOML configuration.
- EXE, COM, BAT, and CMD launch support on Windows.
- Before-game and after-game companion-tool sequencing.
- Optional delays, required-tool behavior, and direct child-process cleanup.
- Guardian/worker process separation and game-process supervision.
- Configuration validation and dry-run command-line modes.
- Session logging.
- Linux quality checks and Windows MSVC build/test coverage in CI.
- Windows build scripts and an isolated Wine smoke test.
- Alpha packaging script and tester instructions.

### Known limitations

- No graphical configuration interface, controller navigation, or notifications.
- No worker restart or cleanup recovery after worker failure.
- Limited real-device validation in GameNative and Winlator.
