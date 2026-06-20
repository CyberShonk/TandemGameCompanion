# Changelog

All notable changes to Tandem Game Companion will be documented here.

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
- Script arguments are not supported.
