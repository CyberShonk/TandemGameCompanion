# Changelog

All notable changes to Tandem Game Companion are documented here.

## Unreleased

### Added

- Generic per-tool preparation recipes with a bounded `wait-for-window` action.
- Process-scoped visible top-level window discovery using exact-title or title-contains matching.
- Windows smoke coverage proving that a same-title window from another process cannot satisfy a
  tool's preparation step.
- Process-scoped `wait-for-control` preparation for visible, enabled standard Win32 descendant
  controls selected by parent-window title, numeric control ID, exact class name, or ID/class AND
  semantics.
- Windows smoke coverage for other-process, other-top-level-window, hidden, disabled, and partial
  ID/class control false matches.
- Process-scoped `select-combo-box-index` preparation for a standard visible, enabled Win32
  `ComboBox`, using zero-based numeric indices, bounded documented messages, result verification,
  and the minimum standard parent selection-change notification.
- Wine smoke coverage for mutation scoping, no-op behavior, out-of-range rejection, ambiguity,
  tool exit, required/optional policy, cleanup, and unchanged wait/guardian behavior.

### Security

- Window and control discovery are restricted to the exact PID Tandem launched. The only control
  mutation is allowlisted standard ComboBox selection by numeric index with runtime-class, item-count,
  before/after result, ambiguity, timeout, and parent-notification checks. Text matching, focus,
  activation, input, arbitrary messages, other control mutations, UI Automation, and image matching
  remain excluded.

## [0.2.0-alpha] - 2026-06-24

### Added

- General-purpose `before_game_wait` modes for native user confirmation and one-shot setup utilities.
- A native Windows OK/Cancel prompt for trainers that must be configured before game launch.
- Lifecycle integration tests for early failures, persistent tools, delayed launches, guardian recovery, and guardian-protocol spoof attempts.
- A user guide, troubleshooting guide, and central documentation index.

### Changed

- Existing configurations remain compatible because omitted `before_game_wait` values default to `none`.
- BAT/CMD game and tool entries preserve validated arguments through Tandem's fixed `cmd.exe` invocation.
- Public documentation now uses a clearer project overview, navigation links, grouped references, and consistent user/developer sections.

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
