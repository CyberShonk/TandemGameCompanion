<div align="center">

# Tandem Game Companion

**Portable companion tool launching for Windows games**

Start a game and its local helper programs as one supervised session.

[![Release](https://img.shields.io/github/v/release/CyberShonk/TandemGameCompanion?include_prereleases&label=release)](https://github.com/CyberShonk/TandemGameCompanion/releases)
[![Continuous integration](https://github.com/CyberShonk/TandemGameCompanion/actions/workflows/ci.yml/badge.svg)](https://github.com/CyberShonk/TandemGameCompanion/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[Releases](https://github.com/CyberShonk/TandemGameCompanion/releases) · [User Guide](docs/user-guide.md) · [Configuration](docs/CONFIGURATION.md) · [Troubleshooting](docs/troubleshooting.md) · [Changelog](CHANGELOG.md)

</div>

---

Tandem Game Companion is a portable Windows launcher for games that need trainers, controller utilities, setup programs, performance tools, scripts, or other local helpers.

Tandem is currently alpha software. Configuration is edited manually and compatibility varies by game, tool, device, and Windows compatibility environment.

## What Tandem does

Tandem can:

- launch EXE, COM, BAT, and CMD entries;
- start tools before or after the game;
- delay tool launches;
- wait for a setup program to finish;
- pause for confirmation before the game starts;
- run ordered preparation steps against standard Win32 windows and controls owned by the tool Tandem launched;
- allow optional tool failures without blocking the game;
- remain active while the configured game process runs;
- close selected tool processes when the game exits; and
- write `Tandem.log` for troubleshooting.

### Tool preparation

Preparation steps are configured under `[[tools.prepare]]`.

| Action | Use |
|---|---|
| `wait-for-window` | Wait for a visible tool window |
| `wait-for-control` | Wait for a visible enabled standard Win32 control |
| `select-combo-box-index` | Select a standard ComboBox item by zero based index |
| `invoke-button` | Invoke a standard push button |
| `set-checkbox-state` | Set a standard automatic checkbox on or off |
| `select-radio-button` | Select a standard automatic radio button |
| `set-edit-text` | Set text in a standard single line editable field |

Preparation is intentionally limited. It does not provide arbitrary keyboard, mouse, image matching, shell command, or generic Windows message automation.

## Quick start

1. Download the latest alpha package from [Releases](https://github.com/CyberShonk/TandemGameCompanion/releases).
2. Extract it beside the game executable.
3. Put companion programs in the included `Tools` folder.
4. Edit `Tandem.toml` with the correct game and tool paths.
5. Configure Windows, Wine, GameNative, or Winlator to launch `TandemGameCompanion.exe` instead of the game executable.
6. Start the game normally.
7. Check `Tandem.log` if anything fails.

A simple portable layout looks like this:

```text
GameFolder/
├── TandemGameCompanion.exe
├── Tandem.toml
├── ExampleGame.exe
└── Tools/
    ├── Trainer.exe
    └── ControllerUtility.exe
```

Start with [`Tandem.example.toml`](Tandem.example.toml). The [User Guide](docs/user-guide.md) covers normal setup and the [Configuration Reference](docs/CONFIGURATION.md) lists every supported field.

## Environments

Tandem is intended for native Windows and Wine based environments, including GameNative and Winlator.

Real device coverage is still limited. Treat compatibility as experimental until the exact game, tool, device, and environment have been tested together.

## Current limitations

- There is no graphical configuration editor.
- Normal configuration requires editing TOML.
- A console window remains visible during normal launch.
- Tandem manages the directly launched game and tool processes. Launchers that replace themselves with another process may need different handling.
- Cleanup applies to the direct tool process Tandem started, not every descendant process.
- GameNative and Winlator compatibility is not guaranteed for every container or device.

## Security

Only configure games, tools, and scripts you trust. Tandem launches them with the current user's normal permissions and does not sandbox them.

Tandem does not request administrator privileges, install services, inject code, download companion tools, or expose an unrestricted shell command field.

See [SECURITY.md](SECURITY.md) for vulnerability reporting and the user facing security boundary.

## Documentation

- [User Guide](docs/user-guide.md)
- [Configuration Reference](docs/CONFIGURATION.md)
- [Troubleshooting](docs/troubleshooting.md)
- [Changelog](CHANGELOG.md)
- [Security Policy](SECURITY.md)

## Contributing

Bug reports, compatibility results, documentation corrections, focused code changes, and reproducible tests are useful. See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Tandem Game Companion is released under the [MIT License](LICENSE).
