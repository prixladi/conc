#! /usr/bin/env bash

set -eu

BIN=$PREFIX/concg

if [[ -z "${PREFIX}" ]]; then
    echo "PREFIX environment must be set"
    exit 1
fi

echo "[GUI] Installing into ${BIN} ..."

sudo cp -f ../../target/release/concg "$BIN"

echo "[GUI] Installed!"
