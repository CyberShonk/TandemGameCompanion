# Tandem Game Companion Testing

[Documentation index](index.md) · [Windows Testing](WINDOWS_TESTING.md) · [Architecture](ARCHITECTURE.md) · [Troubleshooting](troubleshooting.md)

---

Tandem uses Rust unit tests, lifecycle integration tests, repository checks, GitHub Actions CI, and
a Windows/Wine smoke-test harness.

## Local quality checks

```bash
./scripts/check-project.sh
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo test --all-targets --all-features -- --test-threads=1
cargo clippy --all-targets --all-features -- -D warnings
```

## Automated coverage

Unit tests cover:

- CLI parsing;
- TOML defaults;
- wait-mode validation;
- path and file-type validation;
- log escape and overlap protection;
- guardian protocol parsing;
- exit-code mapping; and
- Windows BAT/CMD command construction.

Lifecycle integration tests cover:

- normal game launch;
- user-confirmation waiting;
- tool-exit waiting;
- required-tool failure;
- cleanup after an early game-launch failure;
- persistent tools;
- game exit during a delayed launch;
- child guardian-protocol spoof output; and
- guardian recovery after worker failure.

The documentation does not hardcode a total test count because it changes as coverage grows.

## Continuous integration

`.github/workflows/ci.yml` runs:

### Linux

- repository checks;
- formatting;
- all-target compilation checks;
- unit and integration tests; and
- Clippy with warnings denied.

### Windows MSVC

- Windows-target tests; and
- release compilation.

## Windows and Wine smoke test

```bash
./scripts/test-windows.sh
```

The smoke harness:

1. runs Windows-target tests through Wine;
2. builds the x86-64 Windows MSVC executable;
3. uses an isolated Wine prefix;
4. compiles a small Windows helper;
5. exercises EXE, BAT, and CMD launch paths;
6. verifies BAT/CMD arguments;
7. checks before-game and after-game ordering;
8. confirms logged exit statuses; and
9. simulates worker failure after game startup to verify guardian lifetime and exit status.

See [Windows Testing](WINDOWS_TESTING.md) for prerequisites.

## Manual compatibility matrix

Before a broader release, test at minimum:

- native Windows 10 or Windows 11;
- GameNative on representative Android hardware;
- Winlator or another Android Wine environment;
- 32-bit and 64-bit games;
- games that spawn a replacement process or launcher;
- windowed and fullscreen-like games;
- native-rendering or direct-scanout modes;
- user-confirmation touch and controller mapping;
- required and optional tool failures;
- persistent tools;
- EXE, BAT, and CMD cleanup behavior;
- tools that create descendant processes; and
- worker failure while the game remains active.

Do not claim an environment is supported until it has been tested directly.
