# Windows build and smoke testing

Prerequisites: Rust, `cargo-xwin`, Clang/LLD, Wine, MinGW-w64 GCC, `file`, and `sha256sum`.

```bash
cargo install --locked cargo-xwin
./scripts/build-windows.sh
./scripts/test-windows.sh
```

The build produces `target/windows-release/TandemGameCompanion.exe` and a checksum file whose
record contains only `TandemGameCompanion.exe`, so it is portable between machines.

The Wine smoke test runs Windows-target tests, builds the release executable, exercises EXE,
BAT, and CMD paths (including script arguments), verifies launch order and status logging, and
checks that a simulated worker failure leaves the guardian active until the game exits while
returning a nonzero status.

Manual Windows/GameNative checks must additionally cover the native user-confirmation dialog,
touch/controller focus mapping, Cancel cleanup, a persistent tool, and a full-screen or native-
rendering game obscuring secondary windows after launch.
