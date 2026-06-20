#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$repo_root"

echo "== Private-path check =="
tracked_private="$(git ls-files .private/ 2>/dev/null || true)"
if [[ -n "$tracked_private" ]]; then
    echo "ERROR: private project-context files are tracked:" >&2
    printf '%s\n' "$tracked_private" >&2
    exit 1
fi

echo "== Runtime-file check =="
tracked_runtime="$(git ls-files | grep -E '(^|/)(Tandem\.log|Tandem\.runtime\.toml)$|\.runtime\.toml$' || true)"
if [[ -n "$tracked_runtime" ]]; then
    echo "ERROR: runtime files are tracked:" >&2
    printf '%s\n' "$tracked_runtime" >&2
    exit 1
fi

echo "== Whitespace check =="
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    git diff --check
fi

echo "Repository checks passed."
