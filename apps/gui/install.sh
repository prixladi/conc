#! /bin/env bash

BIN=/usr/local/bin/concg

echo "[GUI] Installing into ${BIN}..."

sudo cp -f ../../target/release/gui "$BIN"

CONFIG="{\"daemon_socket_path\": \"$HOME/.conc/run/conc.sock\"}"
CONFIG_PATH="$HOME/.conc/conf.json"

echo "[GUI] Creating gui config '$CONFIG_PATH'"

echo "$CONFIG" | tee "$CONFIG_PATH" >/dev/null

echo "[GUI] Installed!"
