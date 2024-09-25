#! /bin/env bash

if [ "$EUID" -ne 0 ]; then
  echo "Install requires root permissions!"
  exit
fi

mkdir -p /usr/local/lib/conc

cp ./build/conc /usr/local/lib/conc/conc

ln -sf /usr/local/lib/conc/conc /usr/local/bin
