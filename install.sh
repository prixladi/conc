#! /bin/env bash

if [ "$EUID" -ne 0 ]; then
  echo "Install requires root permissions!"
  exit
fi

mkdir -p /usr/local/lib/conc
mkdir -p /usr/local/lib/conc/run

cp ./build/conc /usr/local/lib/conc/conc

ln -sf /usr/local/lib/conc/conc /usr/local/bin
