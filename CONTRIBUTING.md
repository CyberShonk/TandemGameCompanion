# Contributing to Tandem Game Companion

Tandem Game Companion is alpha software. Keep changes focused, reviewable, and backed by evidence.

## Useful contributions

Useful contributions include:

- reproducible bug reports;
- native Windows, Wine, GameNative, and Winlator compatibility results;
- documentation corrections;
- focused Rust changes; and
- tests for configuration, process lifecycle, and Windows behavior.

Open an issue before beginning a large feature or compatibility change.

## Development checks

Run these before submitting a pull request:

```bash
./scripts/check-project.sh
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo test --all-targets --all-features -- --test-threads=1
cargo clippy --all-targets --all-features -- -D warnings
```

Changes to Windows process or control behavior should also pass the relevant Wine smoke tests in `scripts/`.

## Change requirements

- Keep each commit focused on one coherent responsibility.
- Add or update tests when behavior changes.
- Keep existing `config_version = 1` behavior compatible unless a versioned migration is deliberately introduced.
- Update public documentation when user visible configuration or runtime behavior changes.
- Record direct compatibility results rather than inferring them.
- Do not introduce elevation, injection, automatic downloads, persistence, unrestricted shell commands, or hidden network behavior.

## Documentation changes

Public documentation should describe implemented behavior, current limitations, setup, configuration, troubleshooting, security reporting, and release history. Avoid duplicating the same reference material across several files.

## Bug and compatibility reports

Include the Tandem release or commit, environment version, game and tool names, exact reproduction steps, expected and actual behavior, a sanitized configuration, and the smallest useful part of `Tandem.log`.

Do not upload credentials, copyrighted game files, proprietary third party executables, or personal information that is not needed to reproduce the problem.
