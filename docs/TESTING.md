# Testing

## Local checks

```bash
./scripts/check-project.sh
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

Unit tests cover CLI parsing, TOML defaults, wait-mode validation, path/file validation, log
escape protection, protocol parsing, exit-code mapping, and Windows command construction.
Unix integration tests exercise normal launch, both before-game wait modes, required failures,
early game-launch failure cleanup, persistent tools, interruptible delays, child protocol spoof
output, and guardian recovery.

CI runs Linux checks plus Windows MSVC tests/builds. `./scripts/test-windows.sh` adds Wine smoke
coverage for EXE/BAT/CMD launching, script arguments, sequencing, exit statuses, and guardian
recovery. See [`WINDOWS_TESTING.md`](WINDOWS_TESTING.md).

Real-device validation is still required for native Windows, GameNative, and Winlator,
including replacement-process games and tools that create descendant process trees.
