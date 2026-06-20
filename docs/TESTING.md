# Testing

Tandem has Rust unit tests, repository hygiene checks, GitHub Actions CI, and a Windows/Wine
smoke-test harness.

## Local quality checks

```bash
./scripts/check-project.sh
cargo fmt --check
cargo check --all-targets --all-features
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

The Rust source currently defines 13 test functions. Three script-command tests compile only
for Windows; the remaining tests cover command-line parsing, TOML defaults, path syntax, and
the guardian protocol parser.

## Continuous integration

`.github/workflows/ci.yml` runs:

- Repository checks, formatting, compilation, tests, and Clippy on Linux
- Windows MSVC tests and a release build on `windows-latest`

## Windows/Wine smoke test

```bash
./scripts/test-windows.sh
```

The smoke test:

1. Runs Windows-target Rust tests through Wine.
2. Builds the x86-64 Windows MSVC executable.
3. Uses an isolated Wine prefix.
4. Compiles a small Windows helper program.
5. Exercises EXE, BAT, and CMD launch paths.
6. Checks before-game and after-game ordering.
7. Confirms expected child exit statuses.
8. Simulates worker failure after game startup and checks guardian lifetime.

See [`WINDOWS_TESTING.md`](WINDOWS_TESTING.md) for prerequisites.

## Compatibility validation still required

Before a general release, test at minimum:

- Native Windows 10 or 11
- GameNative on representative Android hardware
- Winlator or another Wine-based Android environment
- 32-bit and 64-bit games
- Games that spawn a replacement process or launcher
- Windowed and fullscreen-like games
- Tool cleanup behavior for EXE, BAT, and CMD entries
- Worker failure while the game remains active
- Controller navigation and notification focus behavior after those features exist

Do not claim an environment is supported until it has been tested directly.
