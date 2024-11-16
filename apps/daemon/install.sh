#! /usr/bin/env bash

set -e

BIN=$PREFIX/concd

if [[ -z "${PREFIX}" ]]; then
    echo "PREFIX environment must be set"
    exit 1
fi

if [[ -z "${SYSTEMD_PREFIX}" ]]; then
    echo "SYSTEMD_PREFIX environment must be set"
    exit 1
fi

echo "[Daemon] Installing into ${BIN} ..."

mkdir -p "$HOME/.conc/run"
sudo cp -f ./build/concd "$BIN"

SERVICE_CONFIG="
[Unit]
Description=Conc service daemon

[Service]
WorkingDirectory=$HOME/.conc/run
ExecStart=$BIN
# optional items below
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target"
SERVICE_CONFIG_LOCATION=$SYSTEMD_PREFIX/concd.service

mkdir -p "$SYSTEMD_PREFIX"
echo "[Daemon] Creating systemd service '$SERVICE_CONFIG_LOCATION'"
echo "$SERVICE_CONFIG" | sudo tee "$SERVICE_CONFIG_LOCATION" >/dev/null

echo "[Daemon] Before you can start using conc you need to start the concd service 'systemctl --user start concd'"
echo "[Daemon] Installed!"
