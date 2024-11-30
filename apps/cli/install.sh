#! /usr/bin/env bash

set -eu

BIN=$PREFIX/concc

if [[ -z "${PREFIX}" ]]; then
    echo "PREFIX environment must be set"
    exit 1
fi

echo "[CLI] Installing into ${BIN} ..."

sudo cp -f ../../target/release/concc "$BIN"

echo "[CLI] Installed!"
