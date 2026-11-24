#! /usr/bin/env bash

set -eu

BIN=$PREFIX/conca

if [[ -z "${PREFIX}" ]]; then
    echo "PREFIX environment must be set"
    exit 1
fi

if [[ -z "${SYSTEMD_PREFIX}" ]]; then
    echo "SYSTEMD_PREFIX environment must be set"
    exit 1
fi

echo "[API] Installing into ${BIN} ..."

sudo cp -f ../../target/release/conca "$BIN"

SERVICE_CONFIG="
[Unit]
Description=Conc HTTP API
After=concd.service

[Service]
ExecStart=$BIN serve
# Change the listen address with: Environment=CONC_API_BIND=127.0.0.1:7777
Restart=always
RestartSec=3

[Install]
WantedBy=default.target"
SERVICE_CONFIG_LOCATION=$SYSTEMD_PREFIX/conca.service

mkdir -p "$SYSTEMD_PREFIX"
echo "[API] Creating systemd service '$SERVICE_CONFIG_LOCATION'"
echo "$SERVICE_CONFIG" | sudo tee "$SERVICE_CONFIG_LOCATION" >/dev/null

echo "[API] Create an API key with 'conca keys create <name>' and start the API with 'systemctl --user start conca'"
echo "[API] Installed!"
