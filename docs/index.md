# Tandem Game Companion Documentation

[Project README](../README.md) · [User Guide](user-guide.md) · [Configuration](CONFIGURATION.md) · [Troubleshooting](troubleshooting.md) · [Changelog](../CHANGELOG.md)

---

This folder contains the user, testing, security, and development documentation for Tandem Game
Companion.

## Where to start

- **Setting up Tandem for the first time:** read the [User Guide](user-guide.md).
- **Editing `Tandem.toml`:** use the [Configuration Reference](CONFIGURATION.md).
- **Something did not launch or close correctly:** check [Troubleshooting](troubleshooting.md).
- **Building or testing the project:** start with [Testing](TESTING.md).
- **Reviewing process supervision or security:** read [Architecture](ARCHITECTURE.md),
  [Guardian and Worker](GUARDIAN_WORKER.md), and the [Security Model](SECURITY_MODEL.md).

## User documentation

| Document | Purpose |
|---|---|
| [User Guide](user-guide.md) | Normal installation, configuration, and launch path |
| [Configuration Reference](CONFIGURATION.md) | Complete TOML schema, validation rules, and wait modes |
| [Troubleshooting](troubleshooting.md) | Common failures, log checks, and likely fixes |
| [Windows Testing](WINDOWS_TESTING.md) | Windows build, Wine smoke test, and manual compatibility checks |
| [Alpha Testing Guide](../packaging/TESTING-INSTRUCTIONS.md) | Instructions included in packaged alpha builds |

## Technical documentation

| Document | Purpose |
|---|---|
| [Architecture](ARCHITECTURE.md) | Runtime structure and source-module responsibilities |
| [Guardian and Worker](GUARDIAN_WORKER.md) | Supervision protocol and recovery boundary |
| [Security Model](SECURITY_MODEL.md) | Trust boundary, restrictions, and user responsibility |
| [Testing](TESTING.md) | Automated checks and manual compatibility matrix |

## Planned interface documentation

| Document | Purpose |
|---|---|
| [UX Requirements](UX_REQUIREMENTS.md) | Planned graphical configuration workflow |
| [Controller Support](CONTROLLER_SUPPORT.md) | Planned focus, input, and accessibility behavior |

## Project files

| Document | Purpose |
|---|---|
| [README](../README.md) | Public project overview and quick start |
| [Changelog](../CHANGELOG.md) | Release history and unreleased changes |
| [Contributing](../CONTRIBUTING.md) | Contribution and validation requirements |
| [Security Policy](../SECURITY.md) | Vulnerability-reporting guidance |
| [License](../LICENSE) | MIT license terms |
