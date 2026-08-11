# Tandem Game Companion Security Policy

## Supported versions

Tandem Game Companion is alpha software. No released version currently has a formal long term security support window.

Security fixes are normally developed against the current `main` branch and included in a later alpha build.

## Reporting a vulnerability

Do not publish sensitive vulnerability details in a public issue.

Use GitHub private vulnerability reporting when it is available for this repository, or contact the repository owner privately through an established project channel.

Include the affected revision or release, reproduction steps, expected and actual behavior, security impact, and the smallest proof of concept needed to reproduce the issue.

Do not include credentials, personal configuration data, copyrighted game files, or proprietary third party executables.

## User security boundary

Tandem launches configured programs with the current user's normal permissions. It does not sandbox games, tools, or scripts. Only configure files you trust.

Tandem does not request administrator privileges, install services or drivers, inject code, download companion tools, create startup persistence, or expose an unrestricted shell command field.

Portable paths are the default. External paths must be enabled explicitly. Tandem also validates configured program paths, working directories, log destinations, script arguments, and supported preparation actions before launch.
