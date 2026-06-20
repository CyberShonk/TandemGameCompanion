# Windows build and smoke testing

Tandem is developed on Linux but targets Windows and Wine-compatible environments.

## Prerequisites

The development container requires:

- Rust stable through `rustup`
- `cargo-xwin`
- Clang and LLD
- Wine
- MinGW-w64 GCC
- `file`
- `sha256sum`

Install the Rust target and `cargo-xwin`:

```bash
rustup target add x86_64-pc-windows-msvc
cargo install --locked cargo-xwin
```

## Build a Windows release executable

```bash
./scripts/build-windows.sh
```

Output:

```text
target/windows-release/TandemGameCompanion.exe
target/windows-release/TandemGameCompanion.exe.sha256
```

## Run the complete Windows smoke test

```bash
./scripts/test-windows.sh
```

The script:

1. Runs Windows-target Rust tests through Wine.
2. Builds the release executable.
3. Creates or reuses an isolated Wine prefix.
4. Compiles a minimal Windows helper program.
5. Tests EXE, BAT, and CMD launch paths.
6. Confirms Tandem stays alive until the mock game exits.
7. Verifies expected events and exit statuses.

Override the Wine prefix when needed:

```bash
WINEPREFIX="$HOME/path/to/another-prefix" ./scripts/test-windows.sh
```
