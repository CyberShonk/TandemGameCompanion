# Security policy

## Supported versions

Tandem Game Companion is early alpha software. No released version currently receives a
formal security-support guarantee.

## Reporting a vulnerability

Do not publish sensitive vulnerability details in a public issue. Use GitHub private
vulnerability reporting when it is available for this repository, or contact the repository
owner privately.

Include:

- The affected revision or release
- Reproduction steps
- Expected and actual behavior
- Security impact
- The smallest proof of concept needed to reproduce the issue

Do not include copyrighted game files, third-party proprietary executables, credentials, or
personal configuration data.

## Security boundary

Tandem does not provide privileges beyond those already held by the current user. A selected
executable or script can perform any action available to that user, so users are responsible
for trusting every file they configure Tandem to launch.

See [`docs/SECURITY_MODEL.md`](docs/SECURITY_MODEL.md) for the implemented restrictions and
known boundaries.
