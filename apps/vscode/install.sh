#! /usr/bin/env bash

set -eu

CODE=${CODE:-code}

if ! command -v "$CODE" > /dev/null; then
    echo "[VSCODE] '$CODE' command not found, set CODE env variable to your VS Code binary"
    exit 1
fi

echo "[VSCODE] Installing extension ..."

"$CODE" --install-extension ./conc.vsix --force

echo "[VSCODE] Installed! Reload VS Code windows to activate it."
