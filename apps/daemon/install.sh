#! /bin/env bash

BIN=/usr/local/bin/concd

echo "[Daemon] Installing into ${BIN}..."

mkdir -p "$HOME/.conc/run"
sudo cp -f ./build/concd "$BIN"

SERVICE_CONFIG="
[Unit]
Description=Conc service daemon

[Service]
User=$USER
WorkingDirectory=$HOME/.conc/run
ExecStart=$BIN
# optional items below
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target"
SERVICE_CONFIG_LOCATION=/usr/lib/systemd/system/concd.service

echo "[Daemon] Creating systemd service '$SERVICE_CONFIG_LOCATION'"
echo "$SERVICE_CONFIG" | sudo tee "$SERVICE_CONFIG_LOCATION" >/dev/null

echo "[Daemon] Before you can start using conc you need to start the concd service 'sudo systemctl start concd'"
echo "[Daemon] Installed!"
