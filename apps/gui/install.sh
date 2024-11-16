#! /usr/bin/env bash

set -e

BIN=$PREFIX/concg

if [[ -z "${PREFIX}" ]]; then
    echo "PREFIX environment must be set"
    exit 1
fi

echo "[GUI] Installing into ${BIN} ..."

sudo cp -f ../../target/release/gui "$BIN"

CONFIG="{
    \"daemon_socket_path\": \"$HOME/.conc/run/conc.sock\",
    \"use_caller_env\": true
}"
CONFIG_PATH="$HOME/.conc/conf.json"

echo "[GUI] Creating gui config '$CONFIG_PATH'"

echo "$CONFIG" | tee "$CONFIG_PATH" >/dev/null

echo "[GUI] Installed!"
