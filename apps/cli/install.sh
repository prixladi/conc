#! /bin/env bash

sudo cp -f ../../target/release/cli /usr/local/bin/concc

CONFIG="{\"daemon_socket_path\": \"$HOME/.conc/run/conc.sock\"}"

echo "$CONFIG" | tee "$HOME/.conc/cli-conf.json" >/dev/null
