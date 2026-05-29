#!/usr/bin/env sh
# Point git at versioned hooks (Gitleaks pre-commit). Run once per clone:
#   ./Scripts/install-githooks.sh
set -e
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT" || exit 1
git config core.hooksPath .githooks
chmod +x .githooks/pre-commit 2>/dev/null || true
echo "Git hooks path set to .githooks (pre-commit runs Gitleaks on staged changes)."
