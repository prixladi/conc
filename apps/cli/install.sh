#! /bin/env bash

BIN=/usr/local/bin/concc

echo "[CLI] Installing into ${BIN}..."

sudo cp -f ../../target/release/cli "$BIN"

CONFIG="{\"daemon_socket_path\": \"$HOME/.conc/run/conc.sock\"}"
CONFIG_PATH="$HOME/.conc/cli-conf.json"

echo "[CLI] Creating cli config '$CONFIG_PATH'"

echo "$CONFIG" | tee "$CONFIG_PATH" >/dev/null

echo "[CLI] Installed!"
