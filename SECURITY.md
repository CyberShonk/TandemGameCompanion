# Tandem Game Companion Security Policy

[Project README](README.md) · [Security Model](docs/SECURITY_MODEL.md) · [Configuration](docs/CONFIGURATION.md)

---

## Supported versions

Tandem Game Companion is alpha software. No released version currently receives a formal security
support window or long-term patch guarantee.

Security fixes are normally developed against the current `main` branch and included in the next
available alpha build.

## Reporting a vulnerability

Do not publish sensitive vulnerability details in a public issue.

Use GitHub private vulnerability reporting when it is available for this repository, or contact the
repository owner privately through an established project channel.

Include:

- affected revision or release;
- reproduction steps;
- expected and actual behavior;
- security impact; and
- the smallest proof of concept required to reproduce the issue.

Do not include copyrighted game files, proprietary third-party executables, credentials, or
personal configuration data.

## Security boundary

Tandem does not provide privileges beyond those already held by the current user. A selected
executable or script can perform any action available to that user, so users are responsible for
trusting every file configured for launch.

Tandem does not sandbox games or tools. See the [Security Model](docs/SECURITY_MODEL.md) for the
implemented restrictions, script rules, status-channel protection, and cleanup boundary.
