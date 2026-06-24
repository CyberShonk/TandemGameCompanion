# Contributing to Tandem Game Companion

[Project README](README.md) · [Documentation](docs/index.md) · [Testing](docs/TESTING.md) · [Security](SECURITY.md)

---

Tandem Game Companion is alpha software. Keep changes focused, reviewable, and easy to validate.
Open an issue before beginning a large feature or compatibility change so its scope can be reviewed
first.

## Useful contributions

Useful contributions include:

- reproducible bug reports;
- GameNative, Winlator, Wine, and native Windows compatibility results;
- documentation corrections;
- focused Rust changes;
- lifecycle and path-validation tests;
- Windows script-handling tests; and
- safer process-cleanup or supervision ideas.

## Development checks

Run these before submitting a pull request:

```bash
./scripts/check-project.sh
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo test --all-targets --all-features -- --test-threads=1
cargo clippy --all-targets --all-features -- -D warnings
```

Windows process, dialog, BAT/CMD, or script changes should also pass:

```bash
./scripts/test-windows.sh
```

See [Testing](docs/TESTING.md) and [Windows Testing](docs/WINDOWS_TESTING.md) for the full validation
scope.

## Change requirements

- Keep each commit focused on one coherent responsibility.
- Add or update tests when behavior changes.
- Update public documentation when configuration or runtime behavior changes.
- Preserve existing configuration compatibility unless a versioned migration is deliberately designed.
- Do not claim GameNative, Winlator, controller, fullscreen, or native-rendering compatibility without direct testing.
- Do not introduce elevation, injection, downloads, persistence, unrestricted shell commands, or hidden network behavior.
- Keep platform-independent logic separate from Windows-specific APIs.
- Document the safety assumptions around every `unsafe` block.
- Record actual validation results rather than inferring them.

## Documentation changes

Use the existing documentation structure:

- `README.md` for the public overview and quick start;
- `docs/user-guide.md` for normal user workflows;
- `docs/CONFIGURATION.md` for exact configuration behavior;
- `docs/troubleshooting.md` for common failures;
- `docs/index.md` as the central documentation map; and
- technical documents for architecture, security, and testing details.

Keep wording direct and human. Avoid duplicating the same detailed reference in several files.

## Bug and compatibility reports

Include:

- Tandem release or commit;
- environment and version;
- device model when applicable;
- game and tool names;
- a sanitized configuration;
- exact reproduction steps;
- expected and actual behavior; and
- the smallest useful section of `Tandem.log`.

Do not upload copyrighted game files, credentials, personal paths that should remain private, or
third-party proprietary executables.
