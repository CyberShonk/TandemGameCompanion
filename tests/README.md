# Tests

Rust unit tests are defined beside the code they protect under `src/`. Unix lifecycle integration tests are in `lifecycle.rs` and cover process ordering, cleanup, exit codes, persistent tools, delayed launches, and guardian isolation.

The `windows-smoke/` directory contains fixtures for the Wine integration test. Generated
executables, logs, event files, and Wine state are written below `target/` or the configured
Wine prefix and are not committed.

Run the platform-independent test suite with:

```bash
cargo test --all-targets --all-features
```

Run the Windows-target integration path from a prepared Linux development environment with:

```bash
./scripts/test-windows.sh
```

See [`../docs/TESTING.md`](../docs/TESTING.md) and
[`../docs/WINDOWS_TESTING.md`](../docs/WINDOWS_TESTING.md).
