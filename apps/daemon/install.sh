#! /bin/env bash

mkdir -p $HOME/.conc/run

sudo cp -f ./build/concd /usr/local/bin/concd

SERVICE="
[Unit]
Description=Conc service daemon

[Service]
User=$USER
WorkingDirectory=$HOME/.conc/run
ExecStart=/usr/local/bin/concd
# optional items below
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target"

echo "$SERVICE" | sudo tee /usr/lib/systemd/system/concd.service >/dev/null