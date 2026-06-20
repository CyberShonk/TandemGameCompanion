# Contributing

Tandem Game Companion is early alpha software. Open an issue before beginning a large change
so its scope and compatibility impact can be reviewed first.

## Development checks

Run these checks before submitting a pull request:

```bash
./scripts/check-project.sh
cargo fmt --check
cargo check --all-targets --all-features
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

Windows process and script changes should also pass:

```bash
./scripts/test-windows.sh
```

## Change requirements

- Keep each change focused on one coherent responsibility.
- Add or update tests when behavior changes.
- Update public documentation when configuration or runtime behavior changes.
- Do not claim GameNative, Winlator, controller, or fullscreen compatibility without direct
  testing.
- Do not introduce elevation, injection, downloads, persistence, unrestricted shell commands,
  or hidden network behavior.
- Keep platform-independent logic separate from Windows-specific APIs.
- Document the safety assumptions around every `unsafe` block.

## Reports

Bug reports should include the Tandem version or commit, environment, relevant configuration
with personal paths removed, reproduction steps, and the smallest useful section of the log.
Do not upload copyrighted game files or third-party proprietary executables.
