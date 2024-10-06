#! /bin/env bash

mkdir -p $HOME/.conc

sudo cp -f ./build/conc /usr/local/bin/conc

SERVICE="
[Unit]
Description=Conc service daemon

[Service]
User=$USER
WorkingDirectory=/home/$USER/.conc
ExecStart=/usr/local/bin/conc
# optional items below
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target"

echo "$SERVICE" | sudo tee /usr/lib/systemd/system/conc.service >/dev/null