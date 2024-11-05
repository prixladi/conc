#! /bin/env bash

BIN=$PREFIX/concc

if [[ -z "${PREFIX}" ]]; then
    echo "PREFIX environment must be set"
    return 1
fi

echo "[CLI] Installing into ${BIN} ..."

sudo cp -f ../../target/release/cli "$BIN"

CONFIG="{\"daemon_socket_path\": \"$HOME/.conc/run/conc.sock\"}"
CONFIG_PATH="$HOME/.conc/conf.json"

echo "[CLI] Creating cli config '$CONFIG_PATH'"

echo "$CONFIG" | tee "$CONFIG_PATH" >/dev/null

echo "[CLI] Installed!"
